use std::path::Path;

use toml_edit::{DocumentMut, Item};

use crate::Error;
use crate::fleet::Fleet;

pub fn clean_manifest_section(
    fleet: &Fleet,
    document: &mut DocumentMut,
    section: &str,
    base: &Path,
    current: &str,
) -> Result<bool, Error> {
    let Some(table) = Fleet::dependency_table_mut(document, section) else {
        return Ok(false);
    };
    let keys: Vec<String> = table.iter().map(|(key, _)| key.to_string()).collect();
    let mut touched = false;
    for key in keys {
        let Some(entry) = table.get_mut(&key).and_then(Item::as_table_like_mut) else {
            continue;
        };
        let Some(path) = entry.get("path").and_then(Item::as_str).map(str::to_string) else {
            continue;
        };
        let Some(dependency) = Fleet::resolve_repo(&fleet.repos, base, &path) else {
            continue;
        };
        if dependency.name == current {
            continue;
        }
        let (is_tag, value) = dependency.tag_or_pushed_rev()?;
        let url = dependency.url.trim_end_matches(".git").to_string();
        entry.remove("path");
        entry.insert("git", toml_edit::value(url));
        if is_tag {
            entry.insert("tag", toml_edit::value(value));
        } else {
            entry.insert("rev", toml_edit::value(value));
        }
        touched = true;
    }
    Ok(touched)
}

#[cfg(test)]
mod tests {
    use super::clean_manifest_section;

    #[test]
    fn test_usage() {
        let _ = clean_manifest_section;
    }
}
