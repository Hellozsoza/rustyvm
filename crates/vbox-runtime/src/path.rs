pub fn join(parts: &[&str]) -> String {
    parts.join("/")
}

pub fn parent(path: &str) -> Option<String> {
    let p = std::path::Path::new(path);
    p.parent().map(|pp| pp.to_string_lossy().into_owned())
}

pub fn file_name(path: &str) -> Option<String> {
    let p = std::path::Path::new(path);
    p.file_name().map(|f| f.to_string_lossy().into_owned())
}

pub fn exists(path: &str) -> bool {
    std::path::Path::new(path).exists()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_join() {
        let p = join(&["a", "b", "c"]);
        assert_eq!(p, "a/b/c");
    }

    #[test]
    fn test_parent() {
        let p = parent("/usr/local/bin");
        assert_eq!(p, Some("/usr/local".to_string()));
    }

    #[test]
    fn test_file_name() {
        let n = file_name("/usr/local/bin");
        assert_eq!(n, Some("bin".to_string()));
    }
}
