use std::path::{Path, PathBuf};

use semver::Version;
use toml_edit::DocumentMut;

use crate::__basic__;
use crate::Error;
use crate::fleet::Fleet;
use crate::git::Git;
use crate::repo::Repo;

pub fn try_from_path(root: &Path) -> Result<Fleet, Error> {
    let mut repos = Vec::new();
    let entries = std::fs::read_dir(root).map_err(|error| io(root, &error))?;
    for entry in entries {
        let dir = entry.map_err(|error| io(root, &error))?.path();
        if !dir.join(".git").exists() {
            continue;
        }
        if let Some(repo) = discover_repo(&dir)? {
            repos.push(repo);
        }
    }
    repos.sort_by(|left, right| left.name.cmp(&right.name));
    let dependencies = dependency_edges(&repos);
    Ok(Fleet {
        root: root.to_path_buf(),
        repos,
        dependencies,
    })
}

// needed helper: one repo from its directory (url, packages, version, version manifest)
fn discover_repo(dir: &Path) -> Result<Option<Repo>, Error> {
    let name = dir
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown")
        .to_string();
    let url = Git::output(dir, &["remote", "get-url", "origin"]).unwrap_or_default();
    let manifests = package_manifests(dir);
    let mut packages = Vec::new();
    let mut workspace: Option<(PathBuf, Version)> = None;
    for manifest in &manifests {
        let Ok(document) = read_manifest(manifest) else {
            continue;
        };
        if let Some(version) = workspace_version(&document) {
            workspace = Some((manifest.clone(), version));
        }
        if let Some(package_name) = document
            .get("package")
            .and_then(toml_edit::Item::as_table_like)
            .and_then(|package| package.get("name"))
            .and_then(toml_edit::Item::as_str)
        {
            packages.push(__basic__::structs::Package {
                name: package_name.to_string(),
                manifest: manifest.clone(),
            });
        }
    }
    if packages.is_empty() && workspace.is_none() {
        return Ok(None);
    }
    let (version_manifest, version) = match workspace {
        Some(workspace_version) => workspace_version,
        None => package_version(&packages, &name)?,
    };
    Ok(Some(Repo {
        name,
        dir: dir.to_path_buf(),
        url,
        packages,
        manifests,
        version_manifest,
        version,
    }))
}

// needed helper: manifests that belong to the repo: its root plus workspace members
fn package_manifests(dir: &Path) -> Vec<PathBuf> {
    let root = dir.join("Cargo.toml");
    let mut out = Vec::new();
    if !root.exists() {
        return out;
    }
    let Ok(document) = read_manifest(&root) else {
        return out;
    };
    out.push(root);
    let members = document
        .get("workspace")
        .and_then(toml_edit::Item::as_table_like)
        .and_then(|workspace| workspace.get("members"))
        .and_then(toml_edit::Item::as_array);
    if let Some(members) = members {
        for member in members.iter().filter_map(toml_edit::Value::as_str) {
            for manifest in expand_member(dir, member) {
                if !out.contains(&manifest) {
                    out.push(manifest);
                }
            }
        }
    }
    out
}

// needed helper: expand one workspace member pattern (literal path or one-level `/*`)
fn expand_member(base: &Path, pattern: &str) -> Vec<PathBuf> {
    if let Some(prefix) = pattern.strip_suffix("/*") {
        let directory = base.join(prefix);
        let mut found = Vec::new();
        let Ok(entries) = std::fs::read_dir(&directory) else {
            return found;
        };
        for entry in entries.flatten() {
            let manifest = entry.path().join("Cargo.toml");
            if manifest.exists() {
                found.push(manifest);
            }
        }
        return found;
    }
    let manifest = base.join(pattern).join("Cargo.toml");
    if manifest.exists() {
        vec![manifest]
    } else {
        Vec::new()
    }
}

// needed helper: parse one manifest with the path attached to any error
fn read_manifest(manifest: &Path) -> Result<DocumentMut, Error> {
    let text = std::fs::read_to_string(manifest).map_err(|error| io(manifest, &error))?;
    text.parse::<DocumentMut>().map_err(|error| Error::Toml {
        path: manifest.display().to_string(),
        message: error.to_string(),
    })
}

// needed helper: `[workspace.package] version` when the repo is a workspace
fn workspace_version(document: &DocumentMut) -> Option<Version> {
    let version = document
        .get("workspace")?
        .as_table_like()?
        .get("package")?
        .as_table_like()?
        .get("version")?
        .as_str()?;
    version.parse::<Version>().ok()
}

// needed helper: version of a single-package repo, or the one shared by all packages
fn package_version(
    packages: &[__basic__::structs::Package],
    repo: &str,
) -> Result<(PathBuf, Version), Error> {
    let Some(first) = packages.first() else {
        return Err(Error::Refused(format!("{repo}: no Cargo package found")));
    };
    let mut found: Option<Version> = None;
    for package in packages {
        let document = read_manifest(&package.manifest)?;
        let version = document
            .get("package")
            .and_then(toml_edit::Item::as_table_like)
            .and_then(|table| table.get("version"))
            .and_then(toml_edit::Item::as_str)
            .ok_or_else(|| {
                Error::Refused(format!(
                    "{repo}: package `{}` has no literal version",
                    package.name
                ))
            })?;
        let parsed = version.parse::<Version>().map_err(|error| {
            Error::Refused(format!(
                "{repo}: package `{}` version `{version}` is not semver: {error}",
                package.name
            ))
        })?;
        match &found {
            None => found = Some(parsed),
            Some(known) if *known == parsed => {}
            Some(known) => {
                return Err(Error::Refused(format!(
                    "{repo}: packages disagree on version ({known} vs {parsed}); the fleet keeps one version per repo"
                )));
            }
        }
    }
    let version = found.unwrap_or_else(|| Version::new(0, 0, 0));
    Ok((first.manifest.clone(), version))
}

// needed helper: repo-level dependency edges, from git pins and local path pins
fn dependency_edges(repos: &[Repo]) -> Vec<__basic__::structs::Dependency> {
    let mut edges: Vec<__basic__::structs::Dependency> = Vec::new();
    for repo in repos {
        for manifest in &repo.manifests {
            let Ok(document) = read_manifest(manifest) else {
                continue;
            };
            let base = manifest.parent().unwrap_or_else(|| Path::new("."));
            for target in dependency_targets(repos, base, &document) {
                if target.name == repo.name {
                    continue;
                }
                let edge = __basic__::structs::Dependency {
                    dependent: repo.name.clone(),
                    dependency: target.name.clone(),
                };
                if !edges.contains(&edge) {
                    edges.push(edge);
                }
            }
        }
    }
    edges
}

// needed helper: the fleet repos one manifest depends on, whichever pin form is on disk
fn dependency_targets<'a>(repos: &'a [Repo], base: &Path, document: &DocumentMut) -> Vec<&'a Repo> {
    let mut targets = Vec::new();
    for section in ["dependencies", "dev-dependencies", "build-dependencies"] {
        if let Some(table) = document
            .get(section)
            .and_then(toml_edit::Item::as_table_like)
        {
            collect_targets(repos, base, table, &mut targets);
        }
    }
    if let Some(table) = document
        .get("workspace")
        .and_then(toml_edit::Item::as_table_like)
        .and_then(|workspace| workspace.get("dependencies"))
        .and_then(toml_edit::Item::as_table_like)
    {
        collect_targets(repos, base, table, &mut targets);
    }
    targets
}

// needed helper: resolve every entry of one dependency table to a fleet repo
fn collect_targets<'a>(
    repos: &'a [Repo],
    base: &Path,
    table: &dyn toml_edit::TableLike,
    targets: &mut Vec<&'a Repo>,
) {
    for (_, item) in table.iter() {
        let Some(entry) = item.as_table_like() else {
            continue;
        };
        let target = entry
            .get("git")
            .and_then(toml_edit::Item::as_str)
            .and_then(|url| repos.iter().find(|repo| repo.matches_url(url)))
            .or_else(|| {
                entry
                    .get("path")
                    .and_then(toml_edit::Item::as_str)
                    .and_then(|path| resolve_repo(repos, base, path))
            });
        if let Some(repo) = target {
            targets.push(repo);
        }
    }
}

// needed helper: the fleet repo a local path points at
fn resolve_repo<'a>(repos: &'a [Repo], base: &Path, path: &str) -> Option<&'a Repo> {
    let target = normalise(&base.join(path));
    repos.iter().find(|repo| {
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

// needed helper: attach the path to an i/o error
fn io(path: &Path, error: &std::io::Error) -> Error {
    Error::Io {
        message: format!("{}: {error}", path.display()),
    }
}

#[cfg(test)]
mod tests {
    use super::try_from_path;

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("README.md"), "empty fleet").unwrap();
        let fleet = try_from_path(dir.path()).unwrap();
        assert!(fleet.root.exists());
        assert_eq!(fleet.dependencies, Vec::new());
    }
}
