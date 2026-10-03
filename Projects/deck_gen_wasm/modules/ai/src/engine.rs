//! `AiEngine`: load one model json5, then run the tool loop on a VFS.

use deck_gen_wasm_conf as wconf;
use deck_gen_wasm_fs::Vfs;
use serde_json::Value;

use crate::conf::{parse_model_conf, AuthKind, CorsKind, ModelConf, ModelKind};
use crate::log::{clip, RunLog};
use crate::tools::{execute_tool, tool_declarations};

/// User-facing request. One prompt, no chat history.
pub enum AiRequest {
    /// Create a new game under `games/<id>/`.
    CreateGame { prompt: String },
    /// Edit `file` inside game root `game` (example `games/poker`).
    EditGame {
        game: String,
        file: String,
        prompt: String,
    },
}

pub(crate) struct ToolCall {
    pub id: String,
    pub name: String,
    pub args: Value,
}

pub(crate) enum MsgRole {
    User,
    Assistant,
    Tool,
}

pub(crate) struct Msg {
    pub role: MsgRole,
    pub text: String,
    pub tool_calls: Vec<ToolCall>,
    pub tool_name: String,
    pub tool_id: String,
}

pub(crate) struct LlmTurn {
    pub text: String,
    pub reasoning: String,
    pub tool_calls: Vec<ToolCall>,
}

/// LLM client built from one `ai-models/<id>.json5`.
#[derive(Debug)]
pub struct AiEngine {
    conf: ModelConf,
}

impl AiEngine {
    /// Read and validate one model file. Empty `api_key` or blocked CORS without
    /// `proxy_url` is an error before any network call.
    pub fn from_conf(vfs: &Vfs, model_path: &str) -> Result<Self, String> {
        let body = vfs
            .read_file(model_path)
            .ok_or_else(|| format!("model config not found: {model_path}"))?;
        let conf = parse_model_conf(body, model_path)?;
        if conf.api_key.trim().is_empty() {
            return Err(format!(
                "api_key is empty in {model_path}. Paste a key into that json5 file."
            ));
        }
        if conf.cors == CorsKind::Blocked && conf.proxy_url.is_empty() {
            return Err(format!(
                "Model '{}' has cors: \"blocked\" and no proxy_url. Set a CORS proxy in the json5, or pick a browser-allowed model.",
                conf.id
            ));
        }
        Ok(Self { conf })
    }

    /// Tool loop. Mutates `vfs` (game files + markdown log). `on_step` is called
    /// after the log is created and after every round so the UI can refresh.
    pub async fn run_loop(
        &self,
        req: AiRequest,
        vfs: &mut Vfs,
        mut on_step: impl FnMut(&Vfs, &str),
    ) -> Result<(), String> {
        let (title, user) = request_text(&req);
        let system = system_prompt(vfs);
        let mut log = RunLog::start(vfs, &self.conf.id, &title, &user)?;
        on_step(vfs, &log.path);

        let mut messages = vec![Msg {
            role: MsgRole::User,
            text: user,
            tool_calls: Vec::new(),
            tool_name: String::new(),
            tool_id: String::new(),
        }];

        for round in 1..=self.conf.max_rounds {
            crate::http::throttle(self.conf.requests_per_second, round > 1).await;
            let turn = self.complete(&system, &messages).await;
            let turn = match turn {
                Ok(t) => t,
                Err(err) => {
                    let _ = log.push(vfs, &format!("## Error\n\n{err}\n"));
                    on_step(vfs, &log.path);
                    return Err(err);
                }
            };

            let mut chunk = format!("## Round {round}\n");
            if !turn.reasoning.trim().is_empty() {
                chunk.push_str("\n### Reasoning\n\n");
                chunk.push_str(&clip(&turn.reasoning));
                chunk.push('\n');
            }
            if !turn.text.trim().is_empty() {
                chunk.push_str("\n### Assistant\n\n");
                chunk.push_str(&clip(&turn.text));
                chunk.push('\n');
            }
            for call in &turn.tool_calls {
                chunk.push_str(&format!(
                    "\n### Tool `{}`\n\n```\n{}\n```\n",
                    call.name,
                    clip(&call.args.to_string())
                ));
            }
            let _ = log.push(vfs, &chunk);

            if turn.tool_calls.is_empty() {
                on_step(vfs, &log.path);
                return Ok(());
            }

            messages.push(Msg {
                role: MsgRole::Assistant,
                text: turn.text,
                tool_calls: turn
                    .tool_calls
                    .iter()
                    .map(|c| ToolCall {
                        id: c.id.clone(),
                        name: c.name.clone(),
                        args: c.args.clone(),
                    })
                    .collect(),
                tool_name: String::new(),
                tool_id: String::new(),
            });

            for call in turn.tool_calls {
                let result = execute_tool(vfs, &call.name, &call.args);
                let _ = log.push(
                    vfs,
                    &format!(
                        "### Result `{}`\n\n```\n{}\n```\n",
                        call.name,
                        clip(&result)
                    ),
                );
                messages.push(Msg {
                    role: MsgRole::Tool,
                    text: result,
                    tool_calls: Vec::new(),
                    tool_name: call.name,
                    tool_id: call.id,
                });
            }
            on_step(vfs, &log.path);
        }

        let err = format!(
            "stopped after {} rounds (max_rounds)",
            self.conf.max_rounds
        );
        let _ = log.push(vfs, &format!("## Error\n\n{err}\n"));
        on_step(vfs, &log.path);
        Err(err)
    }

    async fn complete(&self, system: &str, messages: &[Msg]) -> Result<LlmTurn, String> {
        match self.conf.kind {
            ModelKind::Gemini => crate::gemini::complete(&self.conf, system, messages).await,
            ModelKind::OpenAiCompat => crate::openai::complete(&self.conf, system, messages).await,
        }
    }
}

fn request_text(req: &AiRequest) -> (String, String) {
    match req {
        AiRequest::CreateGame { prompt } => (
            "CreateGame".into(),
            format!(
                "Create a new card game under games/<id>/ following game-help.md.\n\nUser request:\n{prompt}"
            ),
        ),
        AiRequest::EditGame { game, file, prompt } => (
            format!("EditGame {file}"),
            format!(
                "Edit the file `{file}` in the game `{game}`.\nYou may read other files in that game for context. Focus on this file.\n\nUser request:\n{prompt}"
            ),
        ),
    }
}

fn system_prompt(vfs: &Vfs) -> String {
    let help = vfs
        .read_file(wconf::game_help::PATH)
        .unwrap_or("(game-help.md is missing)");
    let tools = tool_declarations();
    format!(
        "You are an agent that creates and edits card games in a virtual filesystem (VFS).\n\
         \n\
         Rules:\n\
         - Read game-help.md (included below) and follow it.\n\
         - Use tools to inspect and change files. Do not assume file contents.\n\
         - After changing game sources you MUST call prepare_html and fix errors it reports.\n\
         - Never write, mkdir, or remove under any `_autogenerated` folder. prepare_html is the only way to create those files.\n\
         - Do not generate PDF or PNG. Cards are HTML views plus JSON5 data.\n\
         - Prefer UTF-8 text files.\n\
         - When finished, stop calling tools and write a short summary.\n\
         \n\
         Tools (use the API tool-calling interface, names: ls, read_file, write_file, mkdir, remove, prepare_html):\n{tools}\n\
         \n\
         <game-help.md>\n{help}\n</game-help.md>\n"
    )
}

pub(crate) fn bearer_headers(conf: &ModelConf) -> Vec<(&'static str, String)> {
    if conf.auth == AuthKind::Bearer {
        vec![("Authorization", format!("Bearer {}", conf.api_key.trim()))]
    } else {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::install::install_ai_defaults;

    #[test]
    fn from_conf_rejects_empty_key() {
        let mut vfs = Vfs::default();
        install_ai_defaults(&mut vfs).unwrap();
        let err = AiEngine::from_conf(&vfs, "ai-models/gemini-2.0-flash.json5").unwrap_err();
        assert!(err.contains("api_key"), "{err}");
    }

    #[test]
    fn from_conf_rejects_blocked_without_proxy() {
        let mut vfs = Vfs::default();
        vfs.put_file(
            "ai-models/deepseek-chat.json5",
            r#"{
                id: "deepseek-chat",
                kind: "openai-compat",
                base_url: "https://api.deepseek.com",
                auth: "bearer",
                api_key: "sk",
                cors: "blocked",
                proxy_url: ""
            }"#
            .into(),
        )
        .unwrap();
        let err = AiEngine::from_conf(&vfs, "ai-models/deepseek-chat.json5").unwrap_err();
        assert!(err.contains("proxy") || err.contains("blocked"), "{err}");
    }
}
