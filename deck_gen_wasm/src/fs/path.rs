//! Workspace paths: `/`-separated, no leading slash, no `.` / `..`.

pub fn parent_path(path: &str) -> String {
    match path.rsplit_once('/') {
        Some((parent, _)) => parent.to_string(),
        None => String::new(),
    }
}

pub fn join_path(parent: &str, name: &str) -> String {
    if parent.is_empty() {
        name.to_string()
    } else {
        format!("{parent}/{name}")
    }
}

pub fn file_name(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

pub fn split_path(path: &str) -> Result<Vec<&str>, String> {
    if path.is_empty() {
        return Ok(Vec::new());
    }
    if path.starts_with('/') || path.contains('\\') {
        return Err("Use paths like folder/file.txt".into());
    }
    let parts: Vec<&str> = path.split('/').collect();
    let invalid = parts
        .iter()
        .any(|part| part.is_empty() || *part == "." || *part == "..");
    if invalid {
        return Err(format!("Invalid path '{path}'"));
    }
    Ok(parts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn join_and_parent_roundtrip() {
        let path = join_path("src", "main.rs");
        assert_eq!(path, "src/main.rs");
        assert_eq!(parent_path(&path), "src");
        assert_eq!(file_name(&path), "main.rs");
    }

    #[test]
    fn root_parent_is_empty() {
        assert_eq!(parent_path("README.md"), "");
        assert_eq!(join_path("", "README.md"), "README.md");
    }

    #[test]
    fn rejects_dot_segments() {
        assert!(split_path("../x").is_err());
        assert!(split_path("a//b").is_err());
        assert!(split_path("/abs").is_err());
    }
}
