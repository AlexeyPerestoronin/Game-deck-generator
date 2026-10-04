//! `AiEngine`: load one model json5, then run the tool loop on a VFS.
//!
//! Agent prompts are markdown in VFS `help/` (`create-game-pt-*.md`, `edit-game-pt-*.md`)
//! chosen by [`AiRequest`] kind and the active UI locale.

use std::sync::Mutex;

use deck_gen_wasm_conf as wconf;
use deck_gen_wasm_fs::Vfs;
use deck_gen_wasm_locale as locale;
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

impl Msg {
    pub(crate) fn user(text: impl Into<String>) -> Self {
        Self {
            role: MsgRole::User,
            text: text.into(),
            tool_calls: Vec::new(),
            tool_name: String::new(),
            tool_id: String::new(),
        }
    }
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
    rpm_stamps: Mutex<Vec<u64>>,
}

impl AiEngine {
    /// Read and validate one model file. Empty `api_key` (after optional
    /// override) or blocked CORS without `proxy_url` is an error before any
    /// network call. Non-empty `api_key_override` replaces json5 `api_key`
    /// for this engine only; the file is not written.
    pub fn from_conf(
        vfs: &Vfs,
        model_path: &str,
        api_key_override: Option<&str>,
    ) -> Result<Self, String> {
        let body = vfs
            .read_file(model_path)
            .ok_or_else(|| format!("model config not found: {model_path}"))?;
        let mut conf = parse_model_conf(body, model_path)?;
        if let Some(key) = api_key_override.map(str::trim).filter(|s| !s.is_empty()) {
            conf.api_key = key.to_string();
        }
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
        Ok(Self {
            conf,
            rpm_stamps: Mutex::new(Vec::new()),
        })
    }

    /// Tool loop. Mutates `vfs` (game files + markdown log). `on_step` is called
    /// after the log is created and after every round so the UI can refresh.
    pub async fn run_loop(
        &self,
        req: AiRequest,
        vfs: &mut Vfs,
        mut on_step: impl FnMut(&Vfs, &str),
    ) -> Result<(), String> {
        // One filled template: system instruction, first user message, and log body
        // (so `ai-models/log` shows the chosen language and nested game-help).
        let (title, user) = request_text(&req, vfs)?;
        let system = user.clone();
        let mut log = RunLog::start(vfs, &self.conf.id, &title, &user)?;
        on_step(vfs, &log.path);

        let mut messages = vec![Msg::user(user)];

        let mut round = 1u32;
        loop {
            if let Some(max) = self.conf.max_rounds {
                if round > max {
                    break;
                }
            }
            crate::http::throttle(self.conf.requests_per_minute, &self.rpm_stamps).await;
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
            round = round.saturating_add(1);
        }

        let stopped = self.conf.max_rounds.unwrap_or(round.saturating_sub(1));
        let err = format!("stopped after {stopped} rounds (max_rounds)");
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

/// One user message using the same URL, body shape, and response parse as [`AiEngine::run_loop`].
pub async fn complete_user_text(conf: &ModelConf, question: &str) -> Result<String, String> {
    let turn = match conf.kind {
        ModelKind::Gemini => crate::gemini::complete_plain(conf, question).await?,
        ModelKind::OpenAiCompat => crate::openai::complete_plain(conf, question).await?,
    };
    let text = turn.text.trim();
    if text.is_empty() {
        let extra = if turn.tool_calls.is_empty() {
            String::new()
        } else {
            format!(" ({} tool call(s))", turn.tool_calls.len())
        };
        return Err(format!("empty model reply{extra}"));
    }
    Ok(turn.text)
}

fn active_locale_code() -> &'static str {
    locale::get_active_locale().as_str()
}

fn prompt_template_path(req: &AiRequest, locale: &str) -> String {
    match req {
        AiRequest::CreateGame { .. } => wconf::ai::create_game_pt(locale),
        AiRequest::EditGame { .. } => wconf::ai::edit_game_pt(locale),
    }
}

fn read_prompt_template(req: &AiRequest, vfs: &Vfs, locale: &str) -> Result<String, String> {
    let path = prompt_template_path(req, locale);
    match vfs.read_file(&path) {
        None => Err(format!("prompt template not found: {path}")),
        Some(body) if crate::install::looks_like_html_document(body) => Err(format!(
            "prompt template is not markdown (HTML shell): {path}"
        )),
        Some(body) => Ok(body.to_string()),
    }
}

fn fill_prompt_template(template: &str, req: &AiRequest, vfs: &Vfs, locale: &str) -> String {
    let help = vfs
        .read_file(&wconf::game_help::path(locale))
        .unwrap_or("(game-help.md is missing)");
    let tools = tool_declarations().to_string();
    let (user_prompt, game, file) = match req {
        AiRequest::CreateGame { prompt } => (prompt.as_str(), "", ""),
        AiRequest::EditGame { game, file, prompt } => {
            (prompt.as_str(), game.as_str(), file.as_str())
        }
    };
    // `{game}` is a prefix of `{game_help}` — replace the longer token first.
    template
        .replace("{game_help}", help)
        .replace("{user_prompt}", user_prompt)
        .replace("{tools}", &tools)
        .replace("{file}", file)
        .replace("{game}", game)
}

fn request_text(req: &AiRequest, vfs: &Vfs) -> Result<(String, String), String> {
    let title = match req {
        AiRequest::CreateGame { .. } => "CreateGame".into(),
        AiRequest::EditGame { file, .. } => format!("EditGame {file}"),
    };
    let locale = active_locale_code();
    let template = read_prompt_template(req, vfs, locale)?;
    Ok((title, fill_prompt_template(&template, req, vfs, locale)))
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

    const EMPTY_GEMINI: &str = r#"{
        id: "gemini-2.0-flash",
        kind: "gemini",
        base_url: "https://example.com",
        auth: "query-key",
        api_key: "",
        cors: "browser"
    }"#;

    fn put_empty_gemini(vfs: &mut Vfs) {
        vfs.put_file("ai-models/gemini-2.0-flash.json5", EMPTY_GEMINI.into())
            .unwrap();
    }

    fn put_pt(vfs: &mut Vfs, locale: &str, create: &str, edit: &str) {
        vfs.put_file(&wconf::ai::create_game_pt(locale), create.into())
            .unwrap();
        vfs.put_file(&wconf::ai::edit_game_pt(locale), edit.into())
            .unwrap();
    }

    #[test]
    fn from_conf_rejects_empty_key() {
        let mut vfs = Vfs::default();
        put_empty_gemini(&mut vfs);
        let err = AiEngine::from_conf(&vfs, "ai-models/gemini-2.0-flash.json5", None).unwrap_err();
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
        let err = AiEngine::from_conf(&vfs, "ai-models/deepseek-chat.json5", None).unwrap_err();
        assert!(err.contains("proxy") || err.contains("blocked"), "{err}");
    }

    #[test]
    fn from_conf_override_fills_empty_json5() {
        let mut vfs = Vfs::default();
        put_empty_gemini(&mut vfs);
        let path = "ai-models/gemini-2.0-flash.json5";
        let before = vfs.read_file(path).unwrap().to_string();
        let engine = AiEngine::from_conf(&vfs, path, Some("from-modal")).unwrap();
        assert_eq!(engine.conf.api_key, "from-modal");
        assert_eq!(vfs.read_file(path).unwrap(), before);
    }

    #[test]
    fn from_conf_both_empty_is_err() {
        let mut vfs = Vfs::default();
        put_empty_gemini(&mut vfs);
        let path = "ai-models/gemini-2.0-flash.json5";
        let err = AiEngine::from_conf(&vfs, path, Some("  ")).unwrap_err();
        assert!(err.contains("api_key"), "{err}");
    }

    #[test]
    fn from_conf_override_wins_over_json5() {
        let mut vfs = Vfs::default();
        vfs.put_file(
            "ai-models/gemini-2.0-flash.json5",
            r#"{
                id: "gemini-2.0-flash",
                kind: "gemini",
                base_url: "https://example.com",
                auth: "query-key",
                api_key: "file-key",
                cors: "browser"
            }"#
            .into(),
        )
        .unwrap();
        let engine =
            AiEngine::from_conf(&vfs, "ai-models/gemini-2.0-flash.json5", Some("modal-key"))
                .unwrap();
        assert_eq!(engine.conf.api_key, "modal-key");
        let body = vfs.read_file("ai-models/gemini-2.0-flash.json5").unwrap();
        assert!(body.contains("file-key"));
        assert!(!body.contains("modal-key"));
    }

    #[test]
    fn from_conf_empty_override_keeps_json5_key() {
        let mut vfs = Vfs::default();
        vfs.put_file(
            "ai-models/gemini-2.0-flash.json5",
            r#"{
                kind: "gemini",
                base_url: "https://example.com",
                api_key: "file-key",
                cors: "browser"
            }"#
            .into(),
        )
        .unwrap();
        let engine =
            AiEngine::from_conf(&vfs, "ai-models/gemini-2.0-flash.json5", Some("")).unwrap();
        assert_eq!(engine.conf.api_key, "file-key");
    }

    fn render(req: &AiRequest, loc: &str, vfs: &Vfs) -> String {
        let template = read_prompt_template(req, vfs, loc).unwrap();
        fill_prompt_template(&template, req, vfs, loc)
    }

    #[test]
    fn create_game_en_has_new_card_game_and_user_prompt() {
        let mut vfs = Vfs::default();
        vfs.put_file(&wconf::game_help::path("en"), "# help body".into())
            .unwrap();
        put_pt(
            &mut vfs,
            "en",
            "Create a new card game\n{user_prompt}\n{game_help}\n{tools}",
            "edit {file}",
        );
        let req = AiRequest::CreateGame {
            prompt: "make uno".into(),
        };
        let out = render(&req, "en", &vfs);
        assert!(!out.trim().is_empty());
        assert!(
            out.contains("new card game"),
            "create en should keep the original task sense: {out}"
        );
        assert!(out.contains("make uno"), "{out}");
        assert!(out.contains("# help body"), "{out}");
        assert!(!out.contains("{user_prompt}"), "{out}");
        assert!(!out.contains("{game_help}"), "{out}");
        assert!(!out.contains("{tools}"), "{out}");
    }

    #[test]
    fn edit_game_substitutes_game_and_file() {
        let mut vfs = Vfs::default();
        put_pt(
            &mut vfs,
            "en",
            "create {user_prompt}",
            "Edit `{file}` in `{game}`.\n{user_prompt}",
        );
        let req = AiRequest::EditGame {
            game: "games/poker".into(),
            file: "games/poker/decks/d/data.json5".into(),
            prompt: "add joker".into(),
        };
        let out = render(&req, "en", &vfs);
        assert!(out.contains("games/poker"), "{out}");
        assert!(out.contains("games/poker/decks/d/data.json5"), "{out}");
        assert!(out.contains("add joker"), "{out}");
        assert!(!out.contains("{file}"), "{out}");
        assert!(!out.contains("{game}"), "{out}");
        assert!(!out.contains("{user_prompt}"), "{out}");
    }

    #[test]
    fn ru_templates_are_not_en() {
        let mut vfs = Vfs::default();
        put_pt(&mut vfs, "en", "EN create {user_prompt}", "EN edit {file}");
        put_pt(&mut vfs, "ru", "RU create {user_prompt}", "RU edit {file}");
        let create = AiRequest::CreateGame {
            prompt: "make uno".into(),
        };
        let edit = AiRequest::EditGame {
            game: "games/poker".into(),
            file: "games/poker/rules.md".into(),
            prompt: "add joker".into(),
        };
        let create_en = render(&create, "en", &vfs);
        let create_ru = render(&create, "ru", &vfs);
        let edit_en = render(&edit, "en", &vfs);
        let edit_ru = render(&edit, "ru", &vfs);
        assert_ne!(create_en, create_ru);
        assert_ne!(edit_en, edit_ru);
        assert!(create_ru.contains("make uno"));
        assert!(edit_ru.contains("games/poker/rules.md"));
    }

    #[test]
    fn fill_reads_game_help_for_locale() {
        let mut vfs = Vfs::default();
        vfs.put_file(&wconf::game_help::path("en"), "# EN HELP".into())
            .unwrap();
        vfs.put_file(&wconf::game_help::path("ru"), "# RU HELP".into())
            .unwrap();
        put_pt(&mut vfs, "en", "{game_help} {user_prompt}", "en {file}");
        put_pt(&mut vfs, "ru", "{game_help} {user_prompt}", "ru {file}");
        let req = AiRequest::CreateGame { prompt: "x".into() };
        let en = render(&req, "en", &vfs);
        let ru = render(&req, "ru", &vfs);
        assert!(en.contains("# EN HELP"), "{en}");
        assert!(ru.contains("# RU HELP"), "{ru}");
        assert!(!en.contains("# RU HELP"), "{en}");
        assert!(!ru.contains("# EN HELP"), "{ru}");
    }

    #[test]
    fn missing_prompt_template_is_err() {
        let vfs = Vfs::default();
        let req = AiRequest::CreateGame { prompt: "x".into() };
        let err = read_prompt_template(&req, &vfs, "en").unwrap_err();
        assert!(err.contains("help/create-game-pt-en.md"), "{err}");
    }

    #[test]
    fn html_shell_prompt_template_is_err() {
        let mut vfs = Vfs::default();
        vfs.put_file(
            &wconf::ai::create_game_pt("en"),
            "<!DOCTYPE html>\n<html></html>".into(),
        )
        .unwrap();
        let req = AiRequest::CreateGame { prompt: "x".into() };
        let err = read_prompt_template(&req, &vfs, "en").unwrap_err();
        assert!(err.contains("HTML"), "{err}");
        assert!(err.contains("help/create-game-pt-en.md"), "{err}");
    }
}
