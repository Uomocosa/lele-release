use crate::Error;
use crate::git::Git;
use crate::repo::Repo;

pub fn refuse_lock_drift(repo: &Repo) -> Result<(), Error> {
    if !repo.dir.join("Cargo.lock").exists() {
        return Ok(());
    }
    let update = std::process::Command::new("cargo")
        .args(["update", "--workspace", "--offline"])
        .current_dir(&repo.dir)
        .output()
        .map_err(|error| Error::Io {
            context: format!("{}: cannot run cargo update", repo.name),
            source: error,
        })?;
    if !update.status.success() {
        return Err(Error::Refused(format!(
            "{}: cargo update --workspace --offline failed\n{}",
            repo.name,
            String::from_utf8_lossy(&update.stderr).trim()
        )));
    }
    let diff =
        Git::command_without_repo_environment(&repo.dir, &["diff", "--quiet", "--", "Cargo.lock"])
            .output()
            .map_err(|error| Error::Io {
                context: format!("{}: cannot run git diff", repo.name),
                source: error,
            })?;
    if diff.status.success() {
        return Ok(());
    }
    if diff.status.code() == Some(1) {
        return Err(Error::Refused(format!(
            "{}: Cargo.lock would change after cargo update --workspace --offline; refusing release",
            repo.name
        )));
    }
    Err(Error::Git {
        repo: repo.dir.display().to_string(),
        command: String::from("diff --quiet -- Cargo.lock"),
        stderr: String::from_utf8_lossy(&diff.stderr).into_owned(),
    })
}

#[cfg(test)]
mod tests {
    use semver::Version;

    use super::refuse_lock_drift;
    use crate::git::Git;
    use crate::testing;

    const MANIFEST: &str = "[package]\nname = \"probe\"\nversion = \"0.0.2\"\nedition = \"2021\"\n";
    const MAIN: &str = "fn main() {}\n";
    const STALE_LOCK: &str = "version = 4\n\n[[package]]\nname = \"probe\"\nversion = \"0.0.1\"\n";

    #[test]
    fn test_usage() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        std::fs::write(dir.join("Cargo.toml"), MANIFEST).unwrap();
        std::fs::create_dir(dir.join("src")).unwrap();
        std::fs::write(dir.join("src/main.rs"), MAIN).unwrap();
        std::fs::write(dir.join("Cargo.lock"), STALE_LOCK).unwrap();
        Git::run(dir, &["init", "-q", "-b", "main"]).unwrap();
        Git::run(dir, &["add", "--", "Cargo.toml", "Cargo.lock"]).unwrap();
        let repo = testing::repo("probe", dir, "", "probe", Version::new(0, 0, 2));

        assert!(refuse_lock_drift(&repo).is_err());

        Git::run(dir, &["add", "--", "Cargo.lock"]).unwrap();
        refuse_lock_drift(&repo).unwrap();
        assert!(Git::run(dir, &["diff", "--quiet", "--", "Cargo.lock"]).is_ok());
    }
}
