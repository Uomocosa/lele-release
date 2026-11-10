use crate::Error;
use crate::repo::Repo;

pub fn untrimmed_status_paths(repo: &Repo) -> Result<Vec<String>, Error> {
    let output = std::process::Command::new("git")
        .args(["status", "--porcelain", "-z"])
        .current_dir(&repo.dir)
        .output()
        .map_err(|error| Error::Io {
            context: format!("{}: cannot read git status", repo.name),
            source: error,
        })?;
    let text = String::from_utf8_lossy(&output.stdout);
    Ok(text
        .split('\0')
        .filter_map(|entry| entry.get(3..))
        .map(str::to_string)
        .collect())
}

#[cfg(test)]
mod tests {
    use semver::Version;

    use super::untrimmed_status_paths;
    use crate::git::Git;
    use crate::repo::Repo;

    #[test]
    fn test_usage() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        std::fs::write(dir.join("tracked.txt"), "one").unwrap();
        Git::run(dir, &["init", "-q", "-b", "main"]).unwrap();
        Git::run(dir, &["add", "tracked.txt"]).unwrap();
        Git::run(
            dir,
            &[
                "-c",
                "user.name=t",
                "-c",
                "user.email=t@t",
                "commit",
                "-qm",
                "init",
            ],
        )
        .unwrap();
        std::fs::write(dir.join("tracked.txt"), "two").unwrap();
        let repo = Repo {
            name: String::from("probe"),
            dir: dir.to_path_buf(),
            url: String::new(),
            packages: Vec::new(),
            manifests: Vec::new(),
            version_manifest: dir.join("Cargo.toml"),
            version: Version::new(0, 0, 1),
        };
        assert_eq!(
            untrimmed_status_paths(&repo).unwrap(),
            vec![String::from("tracked.txt")]
        );
    }
}
