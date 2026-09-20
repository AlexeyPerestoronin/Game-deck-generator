//! JSON-string-aware expansion of `${...}` and `$s{...}`.
//!
//! Deck files are expanded *before* JSON5 parse, so the scanner must know
//! whether it is inside a JSON string: inside, a value is escaped string
//! content; outside, it is a full JSON token. [`cursor`] tracks that state;
//! [`placeholder`] walks `${…}`; [`sticky`] rewrites `$s{…}` to NBSP; [`extract`]
//! slices a `"key": { … }` object out of still-invalid text (for local `vars`).

mod cursor;
mod extract;
mod placeholder;
mod sticky;

pub use extract::extract_json_object_for_key;
pub use placeholder::{lookup_var, substitute_placeholders, Resolve};
pub use sticky::expand_sticky_spans;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn substitutes_outside_string_as_json_token() {
        let out = substitute_placeholders("{\"n\": ${x}}", |_| {
            Ok(Resolve::Value(json!("медбрат")))
        })
        .unwrap();
        assert_eq!(out, "{\"n\": \"медбрат\"}");
    }

    #[test]
    fn keep_leaves_placeholder() {
        let out = substitute_placeholders("{\"n\": ${vars.n}}", |_| Ok(Resolve::Keep)).unwrap();
        assert_eq!(out, "{\"n\": ${vars.n}}");
    }

    #[test]
    fn sticky_span_uses_nbsp_inside_string() {
        let out = expand_sticky_spans("{\"e\": \"$s{5 * X}\"}").unwrap();
        assert!(out.contains('\u{00a0}'), "{out}");
        assert!(!out.contains("$s{"), "{out}");
    }

    #[test]
    fn extracts_vars_object() {
        let raw = r#"{ "vars": { "a": 1 }, "x": ${vars.a} }"#;
        let vars = extract_json_object_for_key(raw, "vars").unwrap().unwrap();
        assert_eq!(vars, json!({"a": 1}));
    }
}
