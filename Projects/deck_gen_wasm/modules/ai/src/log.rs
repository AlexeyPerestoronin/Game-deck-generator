//! Markdown log path + in-run buffer written to VFS every round.

use deck_gen_wasm_conf as wconf;
use deck_gen_wasm_fs::Vfs;

const TOOL_CLIP: usize = 4000;

/// `ai-models/log/<stamp>-<model>.md` with VFS-safe characters (no `:`).
pub fn log_file_name(stamp: &str, model_id: &str) -> String {
    let stamp = safe_segment(stamp);
    let model = safe_segment(model_id);
    format!("{}/{stamp}-{model}.md", wconf::ai::LOG_DIR)
}

fn safe_segment(raw: &str) -> String {
    raw.chars()
        .map(|c| match c {
            ':' | '/' | '\\' | ' ' => '-',
            c if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' || c == 'T' => c,
            _ => '-',
        })
        .collect()
}

pub(crate) fn utc_stamp() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        let d = js_sys::Date::new_0();
        format!(
            "{:04}-{:02}-{:02}T{:02}-{:02}-{:02}",
            d.get_utc_full_year() as i32,
            d.get_utc_month() as u32 + 1,
            d.get_utc_date() as u32,
            d.get_utc_hours() as u32,
            d.get_utc_minutes() as u32,
            d.get_utc_seconds() as u32
        )
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        format!("t{secs}")
    }
}

pub(crate) fn clip(text: &str) -> String {
    if text.len() <= TOOL_CLIP {
        return text.to_string();
    }
    let mut end = TOOL_CLIP.min(text.len());
    while end > 0 && !text.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}… ({} chars)", &text[..end], text.chars().count())
}

pub(crate) struct RunLog {
    pub path: String,
    body: String,
}

impl RunLog {
    pub fn start(vfs: &mut Vfs, model_id: &str, title: &str, user: &str) -> Result<Self, String> {
        let path = log_file_name(&utc_stamp(), model_id);
        let body = format!("# AI run: {model_id}\n\n## Request\n\n{title}\n\n## User\n\n{user}\n");
        vfs.mkdir(wconf::ai::LOG_DIR)?;
        vfs.put_file(&path, body.clone())?;
        Ok(Self { path, body })
    }

    pub fn push(&mut self, vfs: &mut Vfs, chunk: &str) -> Result<(), String> {
        self.body.push('\n');
        self.body.push_str(chunk);
        vfs.put_file(&self.path, self.body.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_name_is_vfs_safe() {
        let name = log_file_name("2026-04-06T12:00:00", "gemini-2.0-flash");
        assert_eq!(
            name,
            "ai-models/log/2026-04-06T12-00-00-gemini-2.0-flash.md"
        );
        assert!(!name.contains(':'));
        assert!(name.starts_with("ai-models/log/"));
        assert!(name.ends_with(".md"));
    }
}
