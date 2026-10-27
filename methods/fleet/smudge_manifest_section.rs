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
    use super::smudge_manifest_section;

    #[test]
    fn test_usage() {
        let _ = smudge_manifest_section;
    }
}
