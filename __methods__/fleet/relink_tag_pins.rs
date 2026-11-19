use std::path::PathBuf;

use crate::Error;
use crate::basic;
use crate::fleet::Fleet;
use crate::repo::Repo;

pub fn relink_tag_pins(
    fleet: &Fleet,
    repo: &Repo,
    dry_run: bool,
) -> Result<Vec<basic::structs::Pin>, Error> {
    let candidates: Vec<basic::structs::Pin> = fleet
        .pins(repo)?
        .into_iter()
        .filter(|pin| {
            matches!(
                pin.form,
                basic::enums::PinForm::Tag(_) | basic::enums::PinForm::Rev(_)
            )
        })
        .collect();
    let mut relinked: Vec<basic::structs::Pin> = Vec::new();
    let mut written: Vec<PathBuf> = Vec::new();
    for pin in &candidates {
        if written.contains(&pin.manifest) {
            continue;
        }
        written.push(pin.manifest.clone());
        let text = std::fs::read_to_string(&pin.manifest)
            .map_err(|error| Fleet::io_error(&pin.manifest, error))?;
        let smudged = fleet.smudge_manifest(&pin.manifest, &text)?;
        if smudged == text {
            continue;
        }
        if !dry_run {
            std::fs::write(&pin.manifest, smudged)
                .map_err(|error| Fleet::io_error(&pin.manifest, error))?;
        }
        relinked.extend(
            candidates
                .iter()
                .filter(|candidate| candidate.manifest == pin.manifest)
                .cloned(),
        );
    }
    if relinked.is_empty() {
        return Ok(relinked);
    }
    let lock = repo.dir.join("Cargo.lock");
    if lock.exists() {
        let text = std::fs::read_to_string(&lock).map_err(|error| Fleet::io_error(&lock, error))?;
        let smudged = fleet.smudge_lock(&lock, &text);
        if smudged != text && !dry_run {
            std::fs::write(&lock, smudged).map_err(|error| Fleet::io_error(&lock, error))?;
        }
    }
    Ok(relinked)
}

#[cfg(test)]
mod tests {
    use semver::Version;

    use super::relink_tag_pins;
    use crate::basic;
    use crate::fleet::Fleet;
    use crate::git::Git;
    use crate::testing;

    #[test]
    fn test_usage() {
        let tmp = tempfile::tempdir().unwrap();
        let dependent_dir = tmp.path().join("dependent");
        let dependent = testing::committed_repo(
            &dependent_dir,
            "dependent",
            Version::new(0, 0, 1),
            Some("v0.0.1"),
        );
        let dependency = testing::committed_repo(
            &tmp.path().join("dep"),
            "dep",
            Version::new(0, 0, 2),
            Some("v0.0.2"),
        );
        std::fs::write(
            dependent_dir.join("Cargo.toml"),
            "[package]\nname = \"dependent\"\nversion = \"0.0.1\"\n\n[dependencies]\ndep = { git = \"https://example.com/dep.git\", tag = \"v0.0.1\" }\n",
        )
        .unwrap();
        let fleet = testing::fleet(tmp.path(), vec![dependent, dependency], Vec::new());
        let repo = fleet
            .repos
            .iter()
            .find(|repo| repo.name == "dependent")
            .unwrap();

        let relinked = relink_tag_pins(&fleet, repo, true).unwrap();

        assert_eq!(relinked.len(), 1);
        let manifest = std::fs::read_to_string(dependent_dir.join("Cargo.toml")).unwrap();
        assert!(manifest.contains("tag = \"v0.0.1\""), "{manifest}");
    }

    #[test]
    fn relinks_a_tag_pin_to_path_and_cleans_to_the_latest_tag() {
        let tmp = tempfile::tempdir().unwrap();
        let dependent_dir = tmp.path().join("dependent");
        let sibling_dir = tmp.path().join("sibling");
        std::fs::create_dir_all(&dependent_dir).unwrap();
        std::fs::create_dir_all(&sibling_dir).unwrap();

        std::fs::write(
            sibling_dir.join("Cargo.toml"),
            "[package]\nname = \"dep\"\nversion = \"0.0.1\"\n",
        )
        .unwrap();
        Git::run(&sibling_dir, &["init", "-q", "-b", "main"]).unwrap();
        Git::run(&sibling_dir, &["add", "--", "Cargo.toml"]).unwrap();
        testing::commit_tag(&sibling_dir, "v0.0.1", "v0.0.1");
        std::fs::write(
            sibling_dir.join("Cargo.toml"),
            "[package]\nname = \"dep\"\nversion = \"0.0.2\"\n",
        )
        .unwrap();
        Git::run(&sibling_dir, &["add", "--", "Cargo.toml"]).unwrap();
        testing::commit_tag(&sibling_dir, "v0.0.2", "v0.0.2");

        let manifest_path = dependent_dir.join("Cargo.toml");
        std::fs::write(
            &manifest_path,
            "[package]\nname = \"dependent\"\nversion = \"0.0.1\"\n\n[dependencies]\ndep = { git = \"https://example.com/dep.git\", tag = \"v0.0.1\" }\n",
        )
        .unwrap();
        std::fs::write(
            dependent_dir.join("Cargo.lock"),
            "version = 3\n\n[[package]]\nname = \"dependent\"\nversion = \"0.0.1\"\n\n[[package]]\nname = \"dep\"\nversion = \"0.0.1\"\nsource = \"git+https://example.com/dep.git?tag=v0.0.1#abc\"\n",
        )
        .unwrap();
        Git::run(&dependent_dir, &["init", "-q", "-b", "main"]).unwrap();

        let sibling = testing::repo(
            "dep",
            &sibling_dir,
            "https://example.com/dep.git",
            "dep",
            Version::new(0, 0, 2),
        );
        let dependent = testing::repo(
            "dependent",
            &dependent_dir,
            "https://example.com/dependent.git",
            "dependent",
            Version::new(0, 0, 1),
        );
        let fleet = Fleet {
            root: tmp.path().to_path_buf(),
            repos: vec![dependent, sibling],
            dependencies: vec![basic::structs::Dependency {
                dependent: String::from("dependent"),
                dependency: String::from("dep"),
            }],
        };
        let dependent = fleet
            .repos
            .iter()
            .find(|candidate| candidate.name == "dependent")
            .unwrap();

        let relinked = relink_tag_pins(&fleet, dependent, false).unwrap();
        assert_eq!(relinked.len(), 1);
        assert_eq!(
            relinked.first().map(|pin| pin.dependency.as_str()),
            Some("dep")
        );

        let manifest = std::fs::read_to_string(&manifest_path).unwrap();
        assert!(manifest.contains("path ="), "{manifest}");
        assert!(!manifest.contains("tag ="), "{manifest}");
        assert!(!manifest.contains("git ="), "{manifest}");

        let cleaned = fleet.clean_manifest(&manifest_path, &manifest).unwrap();
        assert!(cleaned.contains("tag = \"v0.0.2\""), "{cleaned}");
        assert!(!cleaned.contains("path ="), "{cleaned}");

        let lock = std::fs::read_to_string(dependent_dir.join("Cargo.lock")).unwrap();
        assert!(!lock.contains("source ="), "{lock}");
        assert!(
            lock.contains("name = \"dep\"\nversion = \"0.0.2\""),
            "{lock}"
        );
    }
}
