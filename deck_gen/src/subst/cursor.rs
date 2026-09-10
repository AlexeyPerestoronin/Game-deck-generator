//! Byte-index cursor over JSON-like text that tracks whether it is inside a string.

pub struct JsonText<'a> {
    pub raw: &'a str,
    pub i: usize,
    pub in_string: bool,
    pub escaped: bool,
}

impl<'a> JsonText<'a> {
    pub fn new(raw: &'a str) -> Self {
        Self {
            raw,
            i: 0,
            in_string: false,
            escaped: false,
        }
    }

    pub fn remaining(&self) -> bool {
        self.i < self.raw.len()
    }

    pub fn starts_with(&self, token: &str) -> bool {
        self.raw[self.i..].starts_with(token)
    }

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

    pub fn skip_to(&mut self, end: usize) {
        self.i = end;
    }
}

pub fn skip_json_whitespace(raw: &str, mut index: usize) -> usize {
    while index < raw.len() && matches!(raw.as_bytes()[index], b' ' | b'\t' | b'\r' | b'\n') {
        index += 1;
    }
    index
}
