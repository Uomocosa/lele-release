use crate::Error;
use crate::fleet::Fleet;
use crate::git::Git;
use crate::repo::Repo;

pub fn ensure_ready(repo: &Repo, fleet: &Fleet) -> Result<(), Error> {
    let _ = Git::output(&repo.dir, &["fetch", "--quiet", "origin"]);
    repo.run_just_gate()?;
    repo.refuse_unclean_tree(fleet)?;
    let branch = Git::output(&repo.dir, &["rev-parse", "--abbrev-ref", "HEAD"])?;
    if branch != "main" {
        return Err(Error::Refused(format!(
            "{}: on branch `{branch}`, expected `main`",
            repo.name
        )));
    }
    let counts = Git::output(
        &repo.dir,
        &["rev-list", "--left-right", "--count", "origin/main...HEAD"],
    )?;
    let mut numbers = counts.split_whitespace();
    let behind = numbers.next().unwrap_or("?");
    let ahead = numbers.next().unwrap_or("?");
    if behind != "0" || ahead != "0" {
        return Err(Error::Refused(format!(
            "{}: not in sync with origin/main (behind {behind}, ahead {ahead})",
            repo.name
        )));
    }
    repo.latest_tag()?.map_or(Ok(()), |tag| {
        let tagged = tag.trim_start_matches('v');
        if tagged == repo.version.to_string() {
            Ok(())
        } else {
            Err(Error::Refused(format!(
                "{}: manifest version {} does not match tag {tag}",
                repo.name, repo.version
            )))
        }
    })
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::process::Command;

    use semver::Version;

    use super::ensure_ready;
    use crate::testing;

    const CHILD_MARKER: &str = "LELE_RELEASE_TEST_ENSURE_READY";

    #[test]
    fn test_usage() {
        let Ok(dir) = std::env::var(CHILD_MARKER) else {
            let tmp = tempfile::tempdir().unwrap();
            let dir = tmp.path().join("probe");
            let child = Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "methods::repo::ensure_ready::tests::test_usage",
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
            return;
        };
        let repo = testing::committed_repo(Path::new(&dir), "probe", Version::new(0, 0, 1), None);
        let fleet = testing::fleet(Path::new(&dir), Vec::new(), Vec::new());

        let result = ensure_ready(&repo, &fleet);

        assert!(result.is_err(), "a temp checkout has no justfile to gate");
    }
}
