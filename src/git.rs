use std::path::Path;

use atomic_delegate_macros::atomic_delegates;

use crate::Error;

pub struct Git;

#[atomic_delegates]
impl Git {
    pub fn output(dir: &Path, args: &[&str]) -> Result<String, Error> {}
    pub fn run(dir: &Path, args: &[&str]) -> Result<(), Error> {}
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::Git;

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().unwrap();
        assert!(Git::run(dir.path(), &["init", "-q", "-b", "main"]).is_ok());
        assert!(Git::output(dir.path(), &["rev-parse", "--is-inside-work-tree"]).is_ok());
        assert!(Git::output(Path::new("/nonexistent"), &["status"]).is_err());
    }
}
