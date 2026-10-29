use semver::Version;

use crate::Error;
use crate::git::Git;
use crate::repo::Repo;

pub fn verify_staged_lock(repo: &Repo, next: &Version) -> Result<(), Error> {
    if !repo.dir.join("Cargo.lock").exists() {
        return Ok(());
    }
    let staged = Git::output(&repo.dir, &["show", ":Cargo.lock"])?;
    let expected = next.to_string();
    let mut own: Option<String> = None;
    for line in staged.lines() {
        if line == "[[package]]" {
            own = None;
        } else if let Some(name) = line
            .strip_prefix("name = \"")
            .and_then(|rest| rest.strip_suffix('"'))
        {
            if repo.packages.iter().any(|package| package.name == name) {
                own = Some(name.to_string());
            } else {
                own = None;
            }
        } else if let Some(version) = line
            .strip_prefix("version = \"")
            .and_then(|rest| rest.strip_suffix('"'))
            && let Some(current) = own.take()
            && version != expected
        {
            return Err(Error::Refused(format!(
                "{}: staged lock says {current} {version}, release is {next}",
                repo.name
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use semver::Version;

    use super::verify_staged_lock;
    use crate::basic;
    use crate::git::Git;
    use crate::repo::Repo;

    const MANIFEST: &str = "[package]\nname = \"probe\"\nversion = \"0.0.2\"\n";
    const STALE_LOCK: &str = "version = 4\n\n[[package]]\nname = \"other\"\nversion = \"9.9.9\"\n\n[[package]]\nname = \"probe\"\nversion = \"0.0.1\"\n";

    fn repo_at(dir: &Path) -> Repo {
        Repo {
            name: String::from("probe"),
            dir: dir.to_path_buf(),
            url: String::new(),
            packages: vec![basic::structs::Package {
                name: String::from("probe"),
                manifest: dir.join("Cargo.toml"),
            }],
            manifests: vec![dir.join("Cargo.toml")],
            version_manifest: dir.join("Cargo.toml"),
            version: Version::new(0, 0, 2),
        }
    }

    #[test]
    fn test_usage() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        std::fs::write(dir.join("Cargo.toml"), MANIFEST).unwrap();
        std::fs::write(dir.join("Cargo.lock"), STALE_LOCK).unwrap();
        Git::run(dir, &["init", "-q", "-b", "main"]).unwrap();
        let repo = repo_at(dir);
        let next = Version::new(0, 0, 2);

        repo.stage_version_files().unwrap();
        assert!(verify_staged_lock(&repo, &next).is_err());

        repo.bump_lock_version(&next).unwrap();
        repo.stage_version_files().unwrap();
        verify_staged_lock(&repo, &next).unwrap();
        let staged = Git::output(dir, &["show", ":Cargo.lock"]).unwrap();
        assert!(staged.contains("name = \"probe\"\nversion = \"0.0.2\""));
        assert!(staged.contains("name = \"other\"\nversion = \"9.9.9\""));

        let dual = Repo {
            packages: vec![
                basic::structs::Package {
                    name: String::from("probe"),
                    manifest: dir.join("Cargo.toml"),
                },
                basic::structs::Package {
                    name: String::from("probe-aux"),
                    manifest: dir.join("aux/Cargo.toml"),
                },
            ],
            ..repo_at(dir)
        };
        let mixed = "version = 4\n\n[[package]]\nname = \"probe\"\nversion = \"0.0.2\"\n\n[[package]]\nname = \"probe-aux\"\nversion = \"0.0.1\"\n";
        std::fs::write(dir.join("Cargo.lock"), mixed).unwrap();
        dual.stage_version_files().unwrap();
        assert!(verify_staged_lock(&dual, &next).is_err());
        let aligned = mixed.replace(
            "name = \"probe-aux\"\nversion = \"0.0.1\"",
            "name = \"probe-aux\"\nversion = \"0.0.2\"",
        );
        std::fs::write(dir.join("Cargo.lock"), aligned).unwrap();
        dual.stage_version_files().unwrap();
        verify_staged_lock(&dual, &next).unwrap();
    }
}
