use std::path::Path;

use semver::Version;

use crate::Git;
use crate::Repo;
use crate::testing;

pub fn committed_repo(dir: &Path, name: &str, version: Version, tag: Option<&str>) -> Repo {
    std::fs::create_dir_all(dir).unwrap();
    std::fs::write(
        dir.join("Cargo.toml"),
        format!("[package]\nname = \"{name}\"\nversion = \"{version}\"\n"),
    )
    .unwrap();
    Git::run(dir, &["init", "-q", "-b", "main"]).unwrap();
    Git::run(dir, &["config", "user.name", "t"]).unwrap();
    Git::run(dir, &["config", "user.email", "t@t"]).unwrap();
    Git::run(dir, &["add", "--", "Cargo.toml"]).unwrap();
    match tag {
        Some(tag) => testing::commit_tag(dir, tag, tag),
        None => {
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
        }
    }
    testing::repo(
        name,
        dir,
        &format!("https://example.com/{name}.git"),
        name,
        version,
    )
}

#[cfg(test)]
mod tests {
    use semver::Version;

    use super::committed_repo;

    #[test]
    fn test_usage() {
        let tmp = tempfile::tempdir().unwrap();

        let repo = committed_repo(
            &tmp.path().join("probe"),
            "probe",
            Version::new(0, 0, 2),
            Some("v0.0.2"),
        );

        assert_eq!(repo.version, Version::new(0, 0, 2));
        assert_eq!(repo.latest_tag().unwrap(), Some(String::from("v0.0.2")));
    }
}
