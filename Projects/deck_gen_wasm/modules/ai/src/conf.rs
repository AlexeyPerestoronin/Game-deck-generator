//! One JSON5 file = one model (`ai-models/<id>.json5`).
//!
//! Numeric knobs (`requests_per_minute`, `max_rounds`): omit → code default;
//! `-1` → no limit; `requests_per_minute: 0` is an error (not “unlimited”).

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
    /// Vendor page to create a key. Empty/missing in json5 → `None`.
    pub api_key_hosting: Option<String>,
    /// `None` = no throttle (`-1` or omitted).
    pub requests_per_minute: Option<u32>,
    /// `None` = no round cap (`max_rounds: -1`). Omitted json5 → default 12.
    pub max_rounds: Option<u32>,
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
    api_key_hosting: Option<String>,
    requests_per_minute: Option<i64>,
    max_rounds: Option<i64>,
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
    let requests_per_minute = parse_rpm(raw.requests_per_minute, path)?;
    let max_rounds = parse_max_rounds(raw.max_rounds, path)?;
    Ok(ModelConf {
        id,
        label,
        kind,
        base_url: raw.base_url.trim().trim_end_matches('/').to_string(),
        auth,
        api_key: raw.api_key.unwrap_or_default(),
        cors,
        proxy_url: raw.proxy_url.unwrap_or_default().trim().to_string(),
        api_key_hosting: optional_url(raw.api_key_hosting),
        requests_per_minute,
        max_rounds,
    })
}

fn optional_url(raw: Option<String>) -> Option<String> {
    raw.map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

fn parse_rpm(raw: Option<i64>, path: &str) -> Result<Option<u32>, String> {
    match raw {
        None | Some(-1) => Ok(None),
        Some(0) => Err(rpm_zero_err(path)),
        Some(n) => positive_u32(n, path, "requests_per_minute").map(Some),
    }
}

fn parse_max_rounds(raw: Option<i64>, path: &str) -> Result<Option<u32>, String> {
    match raw {
        None => Ok(Some(DEFAULT_MAX_ROUNDS)),
        Some(-1) => Ok(None),
        Some(0) => Ok(Some(DEFAULT_MAX_ROUNDS)),
        Some(n) => positive_u32(n, path, "max_rounds").map(Some),
    }
}

fn positive_u32(n: i64, path: &str, field: &str) -> Result<u32, String> {
    match u32::try_from(n) {
        Ok(v) if v > 0 => Ok(v),
        _ => Err(format!("{path}: {field}: invalid value {n}")),
    }
}

fn rpm_zero_err(path: &str) -> String {
    format!(
        "{path}: requests_per_minute: 0 is not valid / 0 недопустим (not unlimited / это не «без лимита»). Use -1 or omit the field / укажите -1 или опустите поле."
    )
}

/// One `ai-models/*.json5` as shown in the model selector.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelFile {
    /// VFS path (`ai-models/<id>.json5`).
    pub path: String,
    /// Name in the `<select>`.
    pub label: String,
    /// Vendor key page; `None` hides the request button.
    pub api_key_hosting: Option<String>,
}

/// JSON5 files directly under `ai-models/` (not `log/`).
pub fn list_model_files(vfs: &Vfs) -> Vec<ModelFile> {
    let mut out = Vec::new();
    for (name, is_dir) in vfs.children(wconf::ai::DIR) {
        if is_dir || !name.ends_with(".json5") {
            continue;
        }
        let path = format!("{}/{name}", wconf::ai::DIR);
        let (label, api_key_hosting) = vfs
            .read_file(&path)
            .and_then(|body| parse_model_conf(body, &path).ok())
            .map(|c| (c.label, c.api_key_hosting))
            .unwrap_or_else(|| {
                (
                    name.strip_suffix(".json5").unwrap_or(name).to_string(),
                    None,
                )
            });
        out.push(ModelFile {
            path,
            label,
            api_key_hosting,
        });
    }
    out.sort_by(|a, b| a.path.cmp(&b.path));
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
        assert_eq!(c.max_rounds, Some(DEFAULT_MAX_ROUNDS));
        assert_eq!(c.requests_per_minute, None);
        assert_eq!(c.api_key_hosting, None);
    }

    #[test]
    fn parse_openai_bearer_and_rpm() {
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
                requests_per_minute: 1,
                max_rounds: 8
            }"#,
            "ai-models/deepseek-chat.json5",
        )
        .unwrap();
        assert_eq!(c.kind, ModelKind::OpenAiCompat);
        assert_eq!(c.auth, AuthKind::Bearer);
        assert_eq!(c.cors, CorsKind::Blocked);
        assert_eq!(c.base_url, "https://api.deepseek.com");
        assert_eq!(c.requests_per_minute, Some(1));
        assert_eq!(c.max_rounds, Some(8));
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

    #[test]
    fn parse_hosting_url() {
        let c = parse_model_conf(
            r#"{
                kind: "gemini",
                base_url: "https://example.com",
                "api_key_hosting": "https://aistudio.google.com/apikey"
            }"#,
            "ai-models/x.json5",
        )
        .unwrap();
        assert_eq!(
            c.api_key_hosting.as_deref(),
            Some("https://aistudio.google.com/apikey")
        );
    }

    #[test]
    fn parse_hosting_missing_or_empty_is_none() {
        let missing = parse_model_conf(
            r#"{ kind: "gemini", base_url: "http://x" }"#,
            "ai-models/x.json5",
        )
        .unwrap();
        assert_eq!(missing.api_key_hosting, None);

        let empty = parse_model_conf(
            r#"{ kind: "gemini", base_url: "http://x", "api_key_hosting": "" }"#,
            "ai-models/x.json5",
        )
        .unwrap();
        assert_eq!(empty.api_key_hosting, None);

        let spaces = parse_model_conf(
            r#"{ kind: "gemini", base_url: "http://x", "api_key_hosting": "  " }"#,
            "ai-models/x.json5",
        )
        .unwrap();
        assert_eq!(spaces.api_key_hosting, None);
    }

    #[test]
    fn list_model_files_includes_hosting() {
        let mut vfs = Vfs::default();
        vfs.mkdir("ai-models").unwrap();
        vfs.put_file(
            "ai-models/with.json5",
            r#"{ kind: "gemini", base_url: "http://x", label: "With", "api_key_hosting": "https://keys.example" }"#
                .into(),
        )
        .unwrap();
        vfs.put_file(
            "ai-models/without.json5",
            r#"{ kind: "gemini", base_url: "http://x", label: "Without" }"#.into(),
        )
        .unwrap();
        let list = list_model_files(&vfs);
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].path, "ai-models/with.json5");
        assert_eq!(list[0].label, "With");
        assert_eq!(
            list[0].api_key_hosting.as_deref(),
            Some("https://keys.example")
        );
        assert_eq!(list[1].path, "ai-models/without.json5");
        assert_eq!(list[1].api_key_hosting, None);
    }

    #[test]
    fn rpm_one_is_some() {
        let c = parse_model_conf(
            r#"{ kind: "gemini", base_url: "http://x", requests_per_minute: 1 }"#,
            "ai-models/x.json5",
        )
        .unwrap();
        assert_eq!(c.requests_per_minute, Some(1));
    }

    #[test]
    fn rpm_omitted_is_no_throttle() {
        let c = parse_model_conf(
            r#"{ kind: "gemini", base_url: "http://x" }"#,
            "ai-models/x.json5",
        )
        .unwrap();
        assert_eq!(c.requests_per_minute, None);
    }

    #[test]
    fn rpm_minus_one_is_no_throttle() {
        let c = parse_model_conf(
            r#"{ kind: "gemini", base_url: "http://x", requests_per_minute: -1 }"#,
            "ai-models/x.json5",
        )
        .unwrap();
        assert_eq!(c.requests_per_minute, None);
    }

    #[test]
    fn max_rounds_minus_one_is_unlimited() {
        let c = parse_model_conf(
            r#"{ kind: "gemini", base_url: "http://x", max_rounds: -1 }"#,
            "ai-models/x.json5",
        )
        .unwrap();
        assert_eq!(c.max_rounds, None);
    }

    #[test]
    fn max_rounds_omitted_is_default_12() {
        let c = parse_model_conf(
            r#"{ kind: "gemini", base_url: "http://x" }"#,
            "ai-models/x.json5",
        )
        .unwrap();
        assert_eq!(c.max_rounds, Some(DEFAULT_MAX_ROUNDS));
    }

    #[test]
    fn old_rps_name_is_not_rpm() {
        let c = parse_model_conf(
            r#"{ kind: "gemini", base_url: "http://x", requests_per_second: 9 }"#,
            "ai-models/x.json5",
        )
        .unwrap();
        assert_eq!(c.requests_per_minute, None);
    }

    #[test]
    fn rpm_zero_is_localized_error() {
        let err = parse_model_conf(
            r#"{ kind: "gemini", base_url: "http://x", requests_per_minute: 0 }"#,
            "ai-models/x.json5",
        )
        .unwrap_err();
        assert!(err.contains("requests_per_minute"), "{err}");
        assert!(err.contains("0"), "{err}");
        assert!(err.contains("not valid"), "{err}");
        assert!(err.contains("недопустим"), "{err}");
    }
}
