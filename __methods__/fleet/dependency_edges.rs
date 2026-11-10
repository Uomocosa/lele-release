use std::path::Path;

use crate::basic;
use crate::fleet::Fleet;
use crate::repo::Repo;

#[must_use]
pub fn dependency_edges(repos: &[Repo]) -> Vec<basic::structs::Dependency> {
    let mut edges: Vec<basic::structs::Dependency> = Vec::new();
    for repo in repos {
        for manifest in &repo.manifests {
            let Ok(document) = Fleet::read_manifest(manifest) else {
                continue;
            };
            let base = manifest.parent().unwrap_or_else(|| Path::new("."));
            for target in Fleet::dependency_targets(repos, base, &document) {
                if target.name == repo.name {
                    continue;
                }
                let edge = basic::structs::Dependency {
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

#[cfg(test)]
mod tests {
    use super::dependency_edges;

    #[test]
    fn test_usage() {
        let _ = dependency_edges;
    }
}
