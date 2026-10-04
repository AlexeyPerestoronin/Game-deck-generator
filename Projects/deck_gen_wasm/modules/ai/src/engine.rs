//! `AiEngine`: load one model json5, then run the tool loop on a VFS.
//!
//! Agent prompts are bundled markdown (`create-game-pt-*.md`, `edit-game-pt-*.md`)
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

const CREATE_GAME_PT_EN: &str = include_str!("../bundled/create-game-pt-en.md");
const CREATE_GAME_PT_RU: &str = include_str!("../bundled/create-game-pt-ru.md");
const EDIT_GAME_PT_EN: &str = include_str!("../bundled/edit-game-pt-en.md");
const EDIT_GAME_PT_RU: &str = include_str!("../bundled/edit-game-pt-ru.md");

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
        let (title, user) = request_text(&req, vfs);
        let system = user.clone();
        let mut log = RunLog::start(vfs, &self.conf.id, &title, &user)?;
        on_step(vfs, &log.path);

        let mut messages = vec![Msg {
            role: MsgRole::User,
            text: user,
            tool_calls: Vec::new(),
            tool_name: String::new(),
            tool_id: String::new(),
        }];

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

fn active_locale_code() -> &'static str {
    locale::get_active_locale().as_str()
}

fn bundled_prompt_template(req: &AiRequest, locale: &str) -> &'static str {
    let ru = locale.eq_ignore_ascii_case("ru");
    match req {
        AiRequest::CreateGame { .. } => {
            if ru {
                CREATE_GAME_PT_RU
            } else {
                CREATE_GAME_PT_EN
            }
        }
        AiRequest::EditGame { .. } => {
            if ru {
                EDIT_GAME_PT_RU
            } else {
                EDIT_GAME_PT_EN
            }
        }
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

fn request_text(req: &AiRequest, vfs: &Vfs) -> (String, String) {
    let title = match req {
        AiRequest::CreateGame { .. } => "CreateGame".into(),
        AiRequest::EditGame { file, .. } => format!("EditGame {file}"),
    };
    let locale = active_locale_code();
    let template = bundled_prompt_template(req, locale);
    (title, fill_prompt_template(template, req, vfs, locale))
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
        install_ai_defaults(&mut vfs, "en").unwrap();
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
        install_ai_defaults(&mut vfs, "en").unwrap();
        let path = "ai-models/gemini-2.0-flash.json5";
        let before = vfs.read_file(path).unwrap().to_string();
        let engine = AiEngine::from_conf(&vfs, path, Some("from-modal")).unwrap();
        assert_eq!(engine.conf.api_key, "from-modal");
        assert_eq!(vfs.read_file(path).unwrap(), before);
    }

    #[test]
    fn from_conf_both_empty_is_err() {
        let mut vfs = Vfs::default();
        install_ai_defaults(&mut vfs, "en").unwrap();
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
        fill_prompt_template(bundled_prompt_template(req, loc), req, vfs, loc)
    }

    #[test]
    fn create_game_en_has_new_card_game_and_user_prompt() {
        let mut vfs = Vfs::default();
        vfs.put_file(&wconf::game_help::path("en"), "# help body".into())
            .unwrap();
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
        let vfs = Vfs::default();
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
        assert_ne!(CREATE_GAME_PT_EN, CREATE_GAME_PT_RU);
        assert_ne!(EDIT_GAME_PT_EN, EDIT_GAME_PT_RU);
        let vfs = Vfs::default();
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
        assert!(CREATE_GAME_PT_EN.contains("{user_prompt}"));
        assert!(CREATE_GAME_PT_RU.contains("{user_prompt}"));
        assert!(EDIT_GAME_PT_EN.contains("{file}"));
        assert!(EDIT_GAME_PT_RU.contains("{file}"));
    }

    #[test]
    fn fill_reads_game_help_for_locale() {
        let mut vfs = Vfs::default();
        vfs.put_file(&wconf::game_help::path("en"), "# EN HELP".into())
            .unwrap();
        vfs.put_file(&wconf::game_help::path("ru"), "# RU HELP".into())
            .unwrap();
        let req = AiRequest::CreateGame { prompt: "x".into() };
        let en = render(&req, "en", &vfs);
        let ru = render(&req, "ru", &vfs);
        assert!(en.contains("# EN HELP"), "{en}");
        assert!(ru.contains("# RU HELP"), "{ru}");
        assert!(!en.contains("# RU HELP"), "{en}");
        assert!(!ru.contains("# EN HELP"), "{ru}");
    }
}
