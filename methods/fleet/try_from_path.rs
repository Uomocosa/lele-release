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
        repos.push(discover_repo(&dir)?);
    }
    repos.sort_by(|left, right| left.name.cmp(&right.name));
    let dependencies = dependency_edges(&repos)?;
    Ok(Fleet {
        root: root.to_path_buf(),
        repos,
        dependencies,
    })
}

// needed helper: one repo from its directory (url, packages, version, version manifest)
fn discover_repo(dir: &Path) -> Result<Repo, Error> {
    let name = dir
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown")
        .to_string();
    let url = Git::output(dir, &["remote", "get-url", "origin"]).unwrap_or_default();
    let manifests = package_manifests(dir)?;
    let mut packages = Vec::new();
    let mut workspace: Option<(PathBuf, Version)> = None;
    for manifest in &manifests {
        let document = read_manifest(manifest)?;
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
    let (version_manifest, version) = match workspace {
        Some(workspace_version) => workspace_version,
        None => package_version(&packages, &name)?,
    };
    Ok(Repo {
        name,
        dir: dir.to_path_buf(),
        url,
        packages,
        manifests,
        version_manifest,
        version,
    })
}

// needed helper: manifests that belong to the repo: its root plus workspace members
fn package_manifests(dir: &Path) -> Result<Vec<PathBuf>, Error> {
    let root = dir.join("Cargo.toml");
    let mut out = Vec::new();
    if !root.exists() {
        return Ok(out);
    }
    let document = read_manifest(&root)?;
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
    Ok(out)
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

// needed helper: repo-level dependency edges, derived from git pins to sibling repos
fn dependency_edges(repos: &[Repo]) -> Result<Vec<__basic__::structs::Dependency>, Error> {
    let mut edges: Vec<__basic__::structs::Dependency> = Vec::new();
    for repo in repos {
        for manifest in &repo.manifests {
            let document = read_manifest(manifest)?;
            for url in git_dependency_urls(&document) {
                let Some(target) = repos.iter().find(|candidate| candidate.matches_url(&url))
                else {
                    continue;
                };
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
    Ok(edges)
}

// needed helper: git URLs declared in any dependency table of one manifest
fn git_dependency_urls(document: &DocumentMut) -> Vec<String> {
    let mut urls = Vec::new();
    for section in ["dependencies", "dev-dependencies", "build-dependencies"] {
        if let Some(table) = document
            .get(section)
            .and_then(toml_edit::Item::as_table_like)
        {
            for (_, item) in table.iter() {
                if let Some(url) = item
                    .as_table_like()
                    .and_then(|entry| entry.get("git"))
                    .and_then(toml_edit::Item::as_str)
                {
                    urls.push(url.to_string());
                }
            }
        }
    }
    if let Some(table) = document
        .get("workspace")
        .and_then(toml_edit::Item::as_table_like)
        .and_then(|workspace| workspace.get("dependencies"))
        .and_then(toml_edit::Item::as_table_like)
    {
        for (_, item) in table.iter() {
            if let Some(url) = item
                .as_table_like()
                .and_then(|entry| entry.get("git"))
                .and_then(toml_edit::Item::as_str)
            {
                urls.push(url.to_string());
            }
        }
    }
    urls
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
