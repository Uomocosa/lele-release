use crate::Error;
use crate::git::Git;
use crate::repo::Repo;

pub fn refresh_index(repo: &Repo) -> Result<(), Error> {
    let mut paths: Vec<String> = Vec::new();
    if repo.dir.join("Cargo.lock").exists() {
        paths.push(String::from("Cargo.lock"));
    }
    if repo.dir.join("Cargo.toml").exists() {
        paths.push(String::from("Cargo.toml"));
    }
    if paths.is_empty() {
        return Ok(());
    }
    let mut args = vec!["add", "--"];
    args.extend(paths.iter().map(String::as_str));
    Git::run(&repo.dir, &args)
}

#[cfg(test)]
mod tests {
    use semver::Version;

    use super::refresh_index;
    use crate::git::Git;
    use crate::repo::Repo;

    #[test]
    fn test_usage() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        std::fs::write(
            dir.join("Cargo.toml"),
            "[package]\nname = \"probe\"\nversion = \"0.0.1\"\nedition = \"2021\"\n",
        )
        .unwrap();
        std::fs::write(dir.join("Cargo.lock"), "version = 4\n").unwrap();
        std::fs::write(dir.join(".gitattributes"), "Cargo.lock filter=probe\n").unwrap();
        Git::run(dir, &["init", "-q", "-b", "main"]).unwrap();
        Git::run(dir, &["config", "filter.probe.clean", "sed /^#local$/d"]).unwrap();
        Git::run(dir, &["config", "filter.probe.smudge", "cat"]).unwrap();
        Git::run(
            dir,
            &["add", "--", ".gitattributes", "Cargo.toml", "Cargo.lock"],
        )
        .unwrap();
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
        let repo = Repo {
            name: String::from("probe"),
            dir: dir.to_path_buf(),
            url: String::new(),
            packages: Vec::new(),
            manifests: Vec::new(),
            version_manifest: dir.join("Cargo.toml"),
            version: Version::new(0, 0, 1),
        };
        let head_before = Git::output(dir, &["rev-parse", "HEAD"]).unwrap();
        let mut lock = std::fs::read(dir.join("Cargo.lock")).unwrap();
        lock.extend_from_slice(b"#local\n");
        std::fs::write(dir.join("Cargo.lock"), &lock).unwrap();
        refresh_index(&repo).unwrap();
        assert_eq!(repo.untrimmed_status_paths().unwrap(), Vec::<String>::new());
        let head_after = Git::output(dir, &["rev-parse", "HEAD"]).unwrap();
        assert_eq!(head_before, head_after);
    }
}
