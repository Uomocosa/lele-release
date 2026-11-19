use semver::Version;

use crate::Error;
use crate::git::Git;
use crate::repo::Repo;

pub fn commit_tag_and_push(repo: &Repo, next: &Version, tag: &str) -> Result<(), Error> {
    Git::run(
        &repo.dir,
        &[
            "commit",
            "--no-verify",
            "-m",
            &format!("chore(release): {next}"),
        ],
    )?;
    Git::run(
        &repo.dir,
        &["tag", "-a", tag, "-m", &format!("{} {next}", repo.name)],
    )?;
    Git::run(
        &repo.dir,
        &["push", "--no-verify", "--atomic", "origin", "main", tag],
    )
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::process::Command;

    use semver::Version;

    use super::commit_tag_and_push;
    use crate::Git;
    use crate::testing;

    const CHILD_MARKER: &str = "LELE_RELEASE_TEST_COMMIT_TAG_AND_PUSH";
    const TAG: &str = "v0.0.2";

    #[test]
    fn test_usage() {
        let Ok(dir) = std::env::var(CHILD_MARKER) else {
            let tmp = tempfile::tempdir().unwrap();
            let dir = tmp.path().join("probe");
            let child = Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "methods::repo::commit_tag_and_push::tests::test_usage",
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
            assert_eq!(Git::output(&dir, &["tag", "--list", TAG]).unwrap(), TAG);
            return;
        };
        let repo = testing::committed_repo(
            Path::new(&dir),
            "probe",
            Version::new(0, 0, 1),
            Some("v0.0.1"),
        );
        std::fs::write(
            repo.dir.join("Cargo.toml"),
            "[package]\nname = \"probe\"\nversion = \"0.0.2\"\n",
        )
        .unwrap();
        Git::run(&repo.dir, &["add", "--", "Cargo.toml"]).unwrap();
        Git::run(&repo.dir, &["config", "user.name", "t"]).unwrap();
        Git::run(&repo.dir, &["config", "user.email", "t@t"]).unwrap();

        let result = commit_tag_and_push(&repo, &Version::new(0, 0, 2), TAG);

        assert!(result.is_err(), "a missing origin must stop the push");
    }
}
