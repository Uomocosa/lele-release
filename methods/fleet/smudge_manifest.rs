use std::path::Path;

use toml_edit::{DocumentMut, Item};

use crate::Error;
use crate::fleet::Fleet;

pub fn smudge_manifest(fleet: &Fleet, manifest: &Path, input: &str) -> Result<String, Error> {
    let mut document: DocumentMut = input.parse::<DocumentMut>().map_err(|error| Error::Toml {
        path: manifest.to_path_buf(),
        source: error,
    })?;
    let base = manifest
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();
    let mut touched = false;
    for section in [
        "dependencies",
        "dev-dependencies",
        "build-dependencies",
        "workspace.dependencies",
    ] {
        touched |= rewrite(fleet, &mut document, section, &base);
    }
    Ok(if touched {
        document.to_string()
    } else {
        input.to_string()
    })
}

// needed helper: git pins of this manifest become local paths to the sibling checkout
fn rewrite(fleet: &Fleet, document: &mut DocumentMut, section: &str, base: &Path) -> bool {
    let Some(table) = table_mut(document, section) else {
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

// needed helper: mutable access to one dependency table (including workspace deps)
fn table_mut<'a>(
    document: &'a mut DocumentMut,
    section: &str,
) -> Option<&'a mut dyn toml_edit::TableLike> {
    if section == "workspace.dependencies" {
        return document
            .get_mut("workspace")?
            .as_table_like_mut()?
            .get_mut("dependencies")?
            .as_table_like_mut();
    }
    document.get_mut(section)?.as_table_like_mut()
}

#[cfg(test)]
mod tests {
    use super::smudge_manifest;

    #[test]
    fn test_usage() {
        let _ = smudge_manifest;
    }
}
