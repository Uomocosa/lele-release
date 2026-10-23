use std::path::{Path, PathBuf};

use toml_edit::{DocumentMut, Item};

use crate::Error;
use crate::basic;
use crate::fleet::Fleet;
use crate::repo::Repo;

pub fn pins(fleet: &Fleet, repo: &Repo) -> Result<Vec<basic::structs::Pin>, Error> {
    let mut found = Vec::new();
    for manifest in &repo.manifests {
        let document = read(manifest)?;
        for (key, item) in dependency_items(&document) {
            let Some(table) = item.as_table_like() else {
                continue;
            };
            let dependency = table
                .get("package")
                .and_then(Item::as_str)
                .unwrap_or(&key)
                .to_string();
            let Some(form) = form_of(table) else {
                continue;
            };
            if points_at_fleet(fleet, repo, manifest, table, &form) {
                found.push(basic::structs::Pin {
                    manifest: manifest.clone(),
                    dependent: repo.name.clone(),
                    dependency,
                    form,
                });
            }
        }
    }
    Ok(found)
}

// needed helper: read one manifest
fn read(manifest: &Path) -> Result<DocumentMut, Error> {
    let text = std::fs::read_to_string(manifest).map_err(|error| Error::Io {
        context: format!("{}", manifest.display()),
        source: error,
    })?;
    text.parse::<DocumentMut>().map_err(|error| Error::Toml {
        path: manifest.to_path_buf(),
        source: error,
    })
}

// needed helper: dependency entries of every table that can pin a sibling
fn dependency_items(document: &DocumentMut) -> Vec<(String, &Item)> {
    let mut items = Vec::new();
    for section in ["dependencies", "dev-dependencies", "build-dependencies"] {
        if let Some(table) = document.get(section).and_then(Item::as_table_like) {
            for (key, item) in table.iter() {
                items.push((key.to_string(), item));
            }
        }
    }
    if let Some(table) = document
        .get("workspace")
        .and_then(Item::as_table_like)
        .and_then(|workspace| workspace.get("dependencies"))
        .and_then(Item::as_table_like)
    {
        for (key, item) in table.iter() {
            items.push((key.to_string(), item));
        }
    }
    items
}

// needed helper: which pin form an entry uses
fn form_of(table: &dyn toml_edit::TableLike) -> Option<basic::enums::PinForm> {
    if let Some(path) = table.get("path").and_then(Item::as_str) {
        return Some(basic::enums::PinForm::Path(path.to_string()));
    }
    let url = table.get("git").and_then(Item::as_str)?.to_string();
    if let Some(tag) = table.get("tag").and_then(Item::as_str) {
        return Some(basic::enums::PinForm::Tag(tag.to_string()));
    }
    if let Some(rev) = table.get("rev").and_then(Item::as_str) {
        return Some(basic::enums::PinForm::Rev(rev.to_string()));
    }
    if let Some(branch) = table.get("branch").and_then(Item::as_str) {
        return Some(basic::enums::PinForm::Branch(branch.to_string()));
    }
    Some(basic::enums::PinForm::Git(url))
}

// needed helper: does this entry point at one of the fleet's other repos?
fn points_at_fleet(
    fleet: &Fleet,
    repo: &Repo,
    manifest: &Path,
    table: &dyn toml_edit::TableLike,
    form: &basic::enums::PinForm,
) -> bool {
    match form {
        basic::enums::PinForm::Path(path) => {
            let base = manifest.parent().unwrap_or_else(|| Path::new("."));
            resolve(fleet, base, path).is_some_and(|target| target.name != repo.name)
        }
        _ => table.get("git").and_then(Item::as_str).is_some_and(|url| {
            fleet
                .repos
                .iter()
                .any(|candidate| candidate.matches_url(url))
        }),
    }
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
    use super::pins;

    #[test]
    fn test_usage() {
        let _ = pins;
    }
}
