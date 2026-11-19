use std::path::Path;

use crate::Git;

pub fn repo_with_origin(dir: &Path, origin: &str) {
    Git::run(dir, &["init", "-q", "-b", "main"]).unwrap();
    Git::run(dir, &["remote", "add", "origin", origin]).unwrap();
}

#[cfg(test)]
mod tests {
    use crate::Git;

    use super::repo_with_origin;

    #[test]
    fn test_usage() {
        let tmp = tempfile::tempdir().unwrap();
        repo_with_origin(tmp.path(), "https://example.com/probe");
        let origin = Git::output(tmp.path(), &["remote", "get-url", "origin"]).unwrap();
        assert!(origin.contains("https://example.com/probe"), "{origin}");
    }
}
