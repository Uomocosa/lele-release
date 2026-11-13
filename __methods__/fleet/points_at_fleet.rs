use std::path::Path;

use toml_edit::{Item, TableLike};

use crate::basic;
use crate::fleet::Fleet;
use crate::repo::Repo;

pub fn points_at_fleet(
    fleet: &Fleet,
    repo: &Repo,
    manifest: &Path,
    table: &dyn TableLike,
    form: &basic::enums::PinForm,
) -> bool {
    match form {
        basic::enums::PinForm::Path(path) => {
            let base = manifest.parent().unwrap_or_else(|| Path::new("."));
            Fleet::resolve_repo(&fleet.repos, base, path)
                .is_some_and(|target| target.name != repo.name)
        }
        _ => table.get("git").and_then(Item::as_str).is_some_and(|url| {
            fleet
                .repos
                .iter()
                .any(|candidate| candidate.matches_url(url))
        }),
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use semver::Version;
    use toml_edit::DocumentMut;

    use super::points_at_fleet;
    use crate::basic;
    use crate::fleet::Fleet;
    use crate::repo::Repo;

    fn repo(name: &str, url: &str) -> Repo {
        Repo {
            name: name.to_string(),
            dir: PathBuf::new(),
            url: url.to_string(),
            packages: Vec::new(),
            manifests: Vec::new(),
            version_manifest: PathBuf::new(),
            version: Version::new(0, 0, 0),
        }
    }

    #[test]
    fn test_usage() {
        let fleet = Fleet {
            root: PathBuf::new(),
            repos: vec![repo("other", "https://example.com/other.git")],
            dependencies: Vec::new(),
        };
        let me = repo("me", "https://example.com/me.git");
        let document: DocumentMut = "dep = { git = \"https://example.com/other.git\" }"
            .parse()
            .unwrap();
        let table = document
            .get("dep")
            .and_then(|item| item.as_table_like())
            .unwrap();
        let git = basic::enums::PinForm::Git("https://example.com/other.git".to_string());
        assert!(points_at_fleet(
            &fleet,
            &me,
            Path::new("Cargo.toml"),
            table,
            &git
        ));
        let stranger: DocumentMut = "dep = { git = \"https://example.com/x.git\" }"
            .parse()
            .unwrap();
        let table = stranger
            .get("dep")
            .and_then(|item| item.as_table_like())
            .unwrap();
        assert!(!points_at_fleet(
            &fleet,
            &me,
            Path::new("Cargo.toml"),
            table,
            &git
        ));
    }
}
