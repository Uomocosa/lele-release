use std::path::{Path, PathBuf};

use toml_edit::{DocumentMut, Item};

use crate::Error;
use crate::fleet::Fleet;
use crate::git::Git;
use crate::repo::Repo;

pub fn clean_manifest(fleet: &Fleet, manifest: &Path, input: &str) -> Result<String, Error> {
    let mut document: DocumentMut = input.parse::<DocumentMut>().map_err(|error| Error::Toml {
        path: manifest.display().to_string(),
        message: error.to_string(),
    })?;
    let base = manifest
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();
    let current = fleet
        .repos
        .iter()
        .find(|repo| manifest.starts_with(&repo.dir))
        .map(|repo| repo.name.clone())
        .unwrap_or_default();
    let mut touched = false;
    for section in [
        "dependencies",
        "dev-dependencies",
        "build-dependencies",
        "workspace.dependencies",
    ] {
        touched |= rewrite(fleet, &mut document, section, &base, &current)?;
    }
    Ok(if touched {
        document.to_string()
    } else {
        input.to_string()
    })
}

// needed helper: path pins of this manifest become git pins at the dependency's release tag
fn rewrite(
    fleet: &Fleet,
    document: &mut DocumentMut,
    section: &str,
    base: &Path,
    current: &str,
) -> Result<bool, Error> {
    let Some(table) = table_mut(document, section) else {
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
        let Some(dependency) = resolve(fleet, base, &path) else {
            continue;
        };
        if dependency.name == current {
            continue;
        }
        let (is_tag, value) = pinned(dependency)?;
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

// needed helper: the fleet repo a local path points at
fn resolve<'a>(fleet: &'a Fleet, base: &Path, path: &str) -> Option<&'a Repo> {
    let target = normalise(&base.join(path));
    fleet.repos.iter().find(|repo| {
        repo.packages.iter().any(|package| {
            package
                .manifest
                .parent()
                .is_some_and(|dir| normalise(dir) == target)
        })
    })
}

// needed helper: the released tag to pin, or the pushed revision while unreleased
fn pinned(dependency: &Repo) -> Result<(bool, String), Error> {
    if let Some(tag) = dependency.latest_tag()? {
        return Ok((true, tag));
    }
    let head = Git::output(&dependency.dir, &["rev-parse", "HEAD"])?;
    let pushed = Git::output(&dependency.dir, &["rev-parse", "origin/main"]).unwrap_or(head);
    Ok((false, pushed))
}

// needed helper: lexical `.`/`..` normalisation (paths need not exist)
fn normalise(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                out.pop();
            }
            std::path::Component::CurDir => {}
            other => out.push(other),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::clean_manifest;

    #[test]
    fn test_usage() {
        let _ = clean_manifest;
    }
}
