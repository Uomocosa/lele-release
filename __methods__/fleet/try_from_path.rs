use std::path::Path;

use crate::Error;
use crate::fleet::Fleet;

pub fn try_from_path(root: &Path) -> Result<Fleet, Error> {
    let mut repos = Vec::new();
    let entries = std::fs::read_dir(root).map_err(|error| Fleet::io_error(root, error))?;
    for entry in entries {
        let dir = entry.map_err(|error| Fleet::io_error(root, error))?.path();
        if !dir.join(".git").exists() {
            continue;
        }
        if let Some(repo) = Fleet::discover_repo(&dir)? {
            repos.push(repo);
        }
    }
    repos.sort_by(|left, right| left.name.cmp(&right.name));
    let dependencies = Fleet::dependency_edges(&repos);
    Ok(Fleet {
        root: root.to_path_buf(),
        repos,
        dependencies,
    })
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
