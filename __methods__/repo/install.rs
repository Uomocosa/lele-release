use crate::Error;
use crate::repo::Repo;

pub fn install(repo: &Repo) -> Result<(), Error> {
    let output = std::process::Command::new("just")
        .arg("install")
        .current_dir(&repo.dir)
        .output()
        .map_err(|error| Error::Io {
            context: format!("{}: cannot run `just install`", repo.name),
            source: error,
        })?;
    if !output.status.success() {
        return Err(Error::Refused(format!(
            "{}: `just install` failed\n{}",
            repo.name,
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::process::Command;

    use semver::Version;

    use super::install;
    use crate::testing;

    const CHILD_MARKER: &str = "LELE_RELEASE_TEST_INSTALL";

    #[test]
    fn test_usage() {
        let Ok(dir) = std::env::var(CHILD_MARKER) else {
            let tmp = tempfile::tempdir().unwrap();
            let dir = tmp.path().join("probe");
            let child = Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "methods::repo::install::tests::test_usage",
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

        let result = install(&repo);

        assert!(
            result.is_err(),
            "a temp checkout has no justfile to install"
        );
    }
}
