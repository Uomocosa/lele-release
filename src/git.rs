use std::path::Path;
use std::process::Command;

use atomic_delegate_macros::atomic_delegates;

use crate::Error;

pub struct Git;

#[atomic_delegates]
impl Git {
    pub fn output(dir: &Path, args: &[&str]) -> Result<String, Error> {}
    pub fn run(dir: &Path, args: &[&str]) -> Result<(), Error> {}
}

#[atomic_delegates]
impl Git {
    pub(crate) fn command_without_repo_environment(dir: &Path, args: &[&str]) -> Command {}
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::process::Command;

    use super::Git;

    const SIBLING_REPO_PATH_ENV_VAR: &str = "LELE_RELEASE_TEST_SIBLING_REPO";
    const CALLER_REPO_ORIGIN: &str = "https://example.com/caller";
    const SIBLING_REPO_ORIGIN: &str = "https://example.com/sibling";

    fn repo_with_origin(dir: &Path, origin: &str) {
        Git::run(dir, &["init", "-q", "-b", "main"]).unwrap();
        Git::run(dir, &["remote", "add", "origin", origin]).unwrap();
    }

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().unwrap();
        assert!(Git::run(dir.path(), &["init", "-q", "-b", "main"]).is_ok());
        assert!(Git::output(dir.path(), &["rev-parse", "--is-inside-work-tree"]).is_ok());
        assert!(Git::output(Path::new("/nonexistent"), &["status"]).is_err());
    }

    #[test]
    fn does_not_inherit_the_callers_repository_environment() {
        let caller = tempfile::tempdir().unwrap();
        let sibling = tempfile::tempdir().unwrap();
        repo_with_origin(caller.path(), CALLER_REPO_ORIGIN);
        repo_with_origin(sibling.path(), SIBLING_REPO_ORIGIN);

        let child = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "git::tests::reports_the_origin_of_the_sibling_repository",
                "--nocapture",
            ])
            .env("GIT_DIR", caller.path().join(".git"))
            .env("GIT_WORK_TREE", caller.path())
            .env(SIBLING_REPO_PATH_ENV_VAR, sibling.path())
            .output()
            .unwrap();

        let stdout = String::from_utf8_lossy(&child.stdout);
        assert!(child.status.success(), "{stdout}");
        assert!(stdout.contains(SIBLING_REPO_ORIGIN), "{stdout}");
    }

    #[test]
    fn reports_the_origin_of_the_sibling_repository() {
        let Ok(sibling) = std::env::var(SIBLING_REPO_PATH_ENV_VAR) else {
            return;
        };
        let origin = Git::output(Path::new(&sibling), &["remote", "get-url", "origin"]).unwrap();
        println!("sibling origin: {origin}");
    }
}
