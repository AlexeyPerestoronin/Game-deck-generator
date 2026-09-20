//! Byte-index cursor over JSON-like text that tracks whether it is inside a string.
//!
//! Used by placeholder and sticky scanners so `${…}` encoding depends on
//! string context. `take` advances one Unicode scalar and updates `in_string`
//! / `escaped`; it does not interpret JSON beyond quote and backslash.

/// Cursor over `raw` with JSON-string awareness.
pub struct JsonText<'a> {
    /// Source text being scanned.
    pub raw: &'a str,
    /// Byte offset of the next character.
    pub i: usize,
    /// Whether `i` is currently inside a `"…"` string.
    pub in_string: bool,
    /// Whether the previous character in a string was `\`.
    pub escaped: bool,
}

impl<'a> JsonText<'a> {
    /// Start at byte 0, outside a string.
    pub fn new(raw: &'a str) -> Self {
        Self {
            raw,
            i: 0,
            in_string: false,
            escaped: false,
        }
    }

    /// Whether any bytes remain.
    pub fn remaining(&self) -> bool {
        self.i < self.raw.len()
    }

    /// Whether the unread suffix starts with `token`.
    pub fn starts_with(&self, token: &str) -> bool {
        self.raw[self.i..].starts_with(token)
    }

    /// Consume the next char and update string/escape state.
    pub fn take(&mut self) -> char {
        let ch = self.raw[self.i..].chars().next().expect("take at eof");
        self.i += ch.len_utf8();
        if self.in_string {
            if self.escaped {
                self.escaped = false;
            } else if ch == '\\' {
                self.escaped = true;
            } else if ch == '"' {
                self.in_string = false;
            }
        } else if ch == '"' {
            self.in_string = true;
        }
        ch
    }

    /// Jump to `end` without updating string state (caller already parsed a span).
    pub fn skip_to(&mut self, end: usize) {
        self.i = end;
    }
}

/// Advance `index` past space, tab, CR, and LF.
pub fn skip_json_whitespace(raw: &str, mut index: usize) -> usize {
    while index < raw.len() && matches!(raw.as_bytes()[index], b' ' | b'\t' | b'\r' | b'\n') {
        index += 1;
    }
    index
}
