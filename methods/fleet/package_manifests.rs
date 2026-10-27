use std::path::{Path, PathBuf};

use crate::fleet::Fleet;

pub fn package_manifests(dir: &Path) -> Vec<PathBuf> {
    let root = dir.join("Cargo.toml");
    let mut out = Vec::new();
    if !root.exists() {
        return out;
    }
    let Ok(document) = Fleet::read_manifest(&root) else {
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
            for manifest in Fleet::expand_member(dir, member) {
                if !out.contains(&manifest) {
                    out.push(manifest);
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::package_manifests;

    #[test]
    fn test_usage() {
        let _ = package_manifests;
    }
}
