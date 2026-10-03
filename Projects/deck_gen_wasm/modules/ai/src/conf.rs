//! One JSON5 file = one model (`ai-models/<id>.json5`).

use deck_gen_wasm_conf as wconf;
use deck_gen_wasm_fs::{file_name, Vfs};
use serde::Deserialize;

const DEFAULT_MAX_ROUNDS: u32 = 12;

/// How the HTTP client talks to the vendor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelKind {
    Gemini,
    OpenAiCompat,
}

/// Query `?key=` vs `Authorization: Bearer`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthKind {
    QueryKey,
    Bearer,
}

/// Whether the vendor is expected to allow browser CORS.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CorsKind {
    Browser,
    Blocked,
}

/// Parsed model file. Extra JSON5 keys are ignored.
#[derive(Clone, Debug)]
pub struct ModelConf {
    pub id: String,
    pub label: String,
    pub kind: ModelKind,
    pub base_url: String,
    pub auth: AuthKind,
    pub api_key: String,
    pub cors: CorsKind,
    pub proxy_url: String,
    pub requests_per_second: Option<u32>,
    pub max_rounds: u32,
}

#[derive(Deserialize)]
struct RawConf {
    id: Option<String>,
    label: Option<String>,
    kind: String,
    base_url: String,
    auth: Option<String>,
    api_key: Option<String>,
    cors: Option<String>,
    proxy_url: Option<String>,
    requests_per_second: Option<u32>,
    max_rounds: Option<u32>,
}

/// Parse one model JSON5. `path` is used when `id` is missing (file stem).
pub fn parse_model_conf(text: &str, path: &str) -> Result<ModelConf, String> {
    let raw: RawConf =
        json5::from_str(text).map_err(|err| format!("invalid model json5 ({path}): {err}"))?;
    let stem = file_name(path)
        .strip_suffix(".json5")
        .unwrap_or(file_name(path));
    let id = raw
        .id
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| stem.to_string());
    let label = raw
        .label
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| id.clone());
    let kind = match raw.kind.trim() {
        "gemini" => ModelKind::Gemini,
        "openai-compat" => ModelKind::OpenAiCompat,
        other => {
            return Err(format!(
                "{path}: unknown kind '{other}' (use gemini or openai-compat)"
            ))
        }
    };
    let auth = match raw.auth.as_deref().map(str::trim) {
        Some("query-key") | None if kind == ModelKind::Gemini => AuthKind::QueryKey,
        Some("bearer") | None => AuthKind::Bearer,
        Some(other) => {
            return Err(format!(
                "{path}: unknown auth '{other}' (use query-key or bearer)"
            ))
        }
    };
    let cors = match raw.cors.as_deref().map(str::trim) {
        Some("blocked") => CorsKind::Blocked,
        Some("browser") | None => CorsKind::Browser,
        Some(other) => {
            return Err(format!(
                "{path}: unknown cors '{other}' (use browser or blocked)"
            ))
        }
    };
    Ok(ModelConf {
        id,
        label,
        kind,
        base_url: raw.base_url.trim().trim_end_matches('/').to_string(),
        auth,
        api_key: raw.api_key.unwrap_or_default(),
        cors,
        proxy_url: raw.proxy_url.unwrap_or_default().trim().to_string(),
        requests_per_second: raw.requests_per_second.filter(|n| *n > 0),
        max_rounds: raw.max_rounds.filter(|n| *n > 0).unwrap_or(DEFAULT_MAX_ROUNDS),
    })
}

/// JSON5 files directly under `ai-models/` (not `log/`). `(path, label)`.
pub fn list_model_files(vfs: &Vfs) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for (name, is_dir) in vfs.children(wconf::ai::DIR) {
        if is_dir || !name.ends_with(".json5") {
            continue;
        }
        let path = format!("{}/{name}", wconf::ai::DIR);
        let label = vfs
            .read_file(&path)
            .and_then(|body| parse_model_conf(body, &path).ok())
            .map(|c| c.label)
            .unwrap_or_else(|| {
                name.strip_suffix(".json5")
                    .unwrap_or(name)
                    .to_string()
            });
        out.push((path, label));
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_gemini_query_key() {
        let c = parse_model_conf(
            r#"{
                id: "gemini-2.0-flash",
                label: "Gemini 2.0 Flash",
                kind: "gemini",
                base_url: "https://generativelanguage.googleapis.com/v1beta",
                auth: "query-key",
                api_key: "abc",
                cors: "browser"
            }"#,
            "ai-models/gemini-2.0-flash.json5",
        )
        .unwrap();
        assert_eq!(c.id, "gemini-2.0-flash");
        assert_eq!(c.kind, ModelKind::Gemini);
        assert_eq!(c.auth, AuthKind::QueryKey);
        assert_eq!(c.api_key, "abc");
        assert_eq!(c.cors, CorsKind::Browser);
        assert_eq!(c.max_rounds, DEFAULT_MAX_ROUNDS);
        assert_eq!(c.requests_per_second, None);
    }

    #[test]
    fn parse_openai_bearer_and_rps() {
        let c = parse_model_conf(
            r#"{
                id: "deepseek-chat",
                label: "DeepSeek Chat",
                kind: "openai-compat",
                base_url: "https://api.deepseek.com/",
                auth: "bearer",
                api_key: "sk",
                cors: "blocked",
                proxy_url: "https://corsproxy.io/?",
                requests_per_second: 1,
                max_rounds: 8
            }"#,
            "ai-models/deepseek-chat.json5",
        )
        .unwrap();
        assert_eq!(c.kind, ModelKind::OpenAiCompat);
        assert_eq!(c.auth, AuthKind::Bearer);
        assert_eq!(c.cors, CorsKind::Blocked);
        assert_eq!(c.base_url, "https://api.deepseek.com");
        assert_eq!(c.requests_per_second, Some(1));
        assert_eq!(c.max_rounds, 8);
        assert_eq!(c.proxy_url, "https://corsproxy.io/?");
    }

    #[test]
    fn parse_empty_key_is_ok() {
        let c = parse_model_conf(
            r#"{
                kind: "gemini",
                base_url: "https://example.com",
                api_key: ""
            }"#,
            "ai-models/gemini-2.0-flash.json5",
        )
        .unwrap();
        assert!(c.api_key.is_empty());
        assert_eq!(c.id, "gemini-2.0-flash");
        assert_eq!(c.label, "gemini-2.0-flash");
    }

    #[test]
    fn extra_fields_ignored() {
        let c = parse_model_conf(
            r#"{ kind: "gemini", base_url: "http://x", free: true, note: "hi" }"#,
            "ai-models/x.json5",
        )
        .unwrap();
        assert_eq!(c.id, "x");
    }
}
