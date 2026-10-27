use std::path::{Path, PathBuf};

use semver::Version;

use crate::Error;
use crate::basic;
use crate::fleet::Fleet;
use crate::git::Git;
use crate::repo::Repo;

pub fn discover_repo(dir: &Path) -> Result<Option<Repo>, Error> {
    let name = dir
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown")
        .to_string();
    let url = Git::output(dir, &["remote", "get-url", "origin"]).unwrap_or_default();
    let manifests = Fleet::package_manifests(dir);
    let mut packages = Vec::new();
    let mut workspace: Option<(PathBuf, Version)> = None;
    for manifest in &manifests {
        let Ok(document) = Fleet::read_manifest(manifest) else {
            continue;
        };
        if let Some(version) = Fleet::workspace_version(&document) {
            workspace = Some((manifest.clone(), version));
        }
        if let Some(package_name) = document
            .get("package")
            .and_then(toml_edit::Item::as_table_like)
            .and_then(|package| package.get("name"))
            .and_then(toml_edit::Item::as_str)
        {
            packages.push(basic::structs::Package {
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
        None => Fleet::package_version(&packages, &name)?,
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

#[cfg(test)]
mod tests {
    use super::discover_repo;

    #[test]
    fn test_usage() {
        let _ = discover_repo;
    }
}
