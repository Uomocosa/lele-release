use std::path::Path;

use toml_edit::{DocumentMut, Item};

use crate::fleet::Fleet;

pub fn smudge_manifest_section(
    fleet: &Fleet,
    document: &mut DocumentMut,
    section: &str,
    base: &Path,
) -> bool {
    let Some(table) = Fleet::dependency_table_mut(document, section) else {
        return false;
    };
    let keys: Vec<String> = table.iter().map(|(key, _)| key.to_string()).collect();
    let mut touched = false;
    for key in keys {
        let Some(entry) = table.get_mut(&key).and_then(Item::as_table_like_mut) else {
            continue;
        };
        let Some(url) = entry.get("git").and_then(Item::as_str).map(str::to_string) else {
            continue;
        };
        let Some(dependency) = fleet.repos.iter().find(|repo| repo.matches_url(&url)) else {
            continue;
        };
        let name = entry
            .get("package")
            .and_then(Item::as_str)
            .unwrap_or(&key)
            .to_string();
        let Some(package) = dependency
            .packages
            .iter()
            .find(|package| package.name == name)
        else {
            continue;
        };
        let Some(dir) = package.manifest.parent() else {
            continue;
        };
        if !dir.exists() {
            continue;
        }
        let Some(relative) = pathdiff::diff_paths(dir, base) else {
            continue;
        };
        entry.remove("git");
        entry.remove("tag");
        entry.remove("rev");
        entry.remove("branch");
        entry.insert("path", toml_edit::value(relative.display().to_string()));
        touched = true;
    }
    touched
}

#[cfg(test)]
mod tests {
    use semver::Version;
    use toml_edit::DocumentMut;

    use super::smudge_manifest_section;
    use crate::testing;

    #[test]
    fn test_usage() {
        let tmp = tempfile::tempdir().unwrap();
        let dependent_dir = tmp.path().join("dependent");
        let dep_dir = tmp.path().join("dep");
        std::fs::create_dir_all(&dependent_dir).unwrap();
        std::fs::create_dir_all(&dep_dir).unwrap();
        std::fs::write(
            dep_dir.join("Cargo.toml"),
            "[package]\nname = \"dep\"\nversion = \"0.0.2\"\n",
        )
        .unwrap();
        let dependent = testing::repo(
            "dependent",
            &dependent_dir,
            "https://example.com/dependent.git",
            "dependent",
            Version::new(0, 0, 1),
        );
        let dependency = testing::repo(
            "dep",
            &dep_dir,
            "https://example.com/dep.git",
            "dep",
            Version::new(0, 0, 2),
        );
        let fleet = testing::fleet(tmp.path(), vec![dependent, dependency], Vec::new());
        let mut document: DocumentMut =
            "[dependencies]\ndep = { git = \"https://example.com/dep.git\", tag = \"v0.0.2\" }\n"
                .parse()
                .unwrap();

        let touched =
            smudge_manifest_section(&fleet, &mut document, "dependencies", &dependent_dir);

        assert!(touched);
        let text = document.to_string();
        assert!(text.contains("path = \"../dep\""), "{text}");
        assert!(!text.contains("tag ="), "{text}");
    }
}
