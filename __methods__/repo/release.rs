use semver::Version;

use crate::Error;
use crate::repo::Repo;

pub fn release(repo: &Repo, next: &Version) -> Result<(), Error> {
    let tag = format!("v{next}");
    repo.write_version(next)?;
    repo.bump_lock_version(next)?;
    repo.stage_version_files()?;
    repo.verify_staged_lock(next)?;
    repo.refuse_lock_drift()?;
    match repo.commit_tag_and_push(next, &tag) {
        Ok(()) => repo.refresh_index(),
        Err(error) => {
            repo.rollback_to_origin(&tag);
            Err(error)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::process::Command;

    use semver::Version;

    use super::release;
    use crate::Git;
    use crate::testing;

    const CHILD_MARKER: &str = "LELE_RELEASE_TEST_RELEASE";
    const TAG: &str = "v0.0.3";

    #[test]
    fn test_usage() {
        let Ok(dir) = std::env::var(CHILD_MARKER) else {
            let tmp = tempfile::tempdir().unwrap();
            let dir = tmp.path().join("probe");
            let child = Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "methods::repo::release::tests::test_usage",
                    "--nocapture",
                ])
                .env(CHILD_MARKER, &dir)
                .env("HOME", tmp.path())
                .env("XDG_CONFIG_HOME", tmp.path().join(".config"))
                .env("GIT_CONFIG_GLOBAL", tmp.path().join(".gitconfig"))
                .output()
                .unwrap();
            let stdout = String::from_utf8_lossy(&child.stdout);
            assert!(child.status.success(), "{stdout}");
            assert_eq!(Git::output(&dir, &["tag", "--list", TAG]).unwrap(), "");
            return;
        };
        let repo = testing::committed_repo(
            Path::new(&dir),
            "probe",
            Version::new(0, 0, 2),
            Some("v0.0.1"),
        );
        Git::run(&repo.dir, &["config", "user.name", "t"]).unwrap();
        Git::run(&repo.dir, &["config", "user.email", "t@t"]).unwrap();

        let result = release(&repo, &Version::new(0, 0, 3));

        assert!(
            result.is_err(),
            "a missing origin stops the release after the rollback"
        );
    }
}
