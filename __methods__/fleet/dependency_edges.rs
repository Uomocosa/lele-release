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
    use semver::Version;

    use super::dependency_edges;
    use crate::testing;

    #[test]
    fn test_usage() {
        let tmp = tempfile::tempdir().unwrap();
        let app_dir = tmp.path().join("app");
        let lib_dir = tmp.path().join("lib");
        std::fs::create_dir_all(&app_dir).unwrap();
        std::fs::create_dir_all(&lib_dir).unwrap();
        std::fs::write(
            app_dir.join("Cargo.toml"),
            "[package]\nname = \"app\"\nversion = \"0.0.1\"\n\n[dependencies]\nlib = { path = \"../lib\" }\n",
        )
        .unwrap();
        std::fs::write(
            lib_dir.join("Cargo.toml"),
            "[package]\nname = \"lib\"\nversion = \"0.0.1\"\n",
        )
        .unwrap();
        let repos = vec![
            testing::repo(
                "app",
                &app_dir,
                "https://example.com/app.git",
                "app",
                Version::new(0, 0, 1),
            ),
            testing::repo(
                "lib",
                &lib_dir,
                "https://example.com/lib.git",
                "lib",
                Version::new(0, 0, 1),
            ),
        ];

        let edges = dependency_edges(&repos);

        assert_eq!(edges.len(), 1);
        assert_eq!(
            edges
                .first()
                .map(|edge| (edge.dependent.as_str(), edge.dependency.as_str())),
            Some(("app", "lib"))
        );
    }
}
