use toml_edit::Item;

use crate::Error;
use crate::basic;
use crate::fleet::Fleet;
use crate::repo::Repo;

pub fn pins(fleet: &Fleet, repo: &Repo) -> Result<Vec<basic::structs::Pin>, Error> {
    let mut found = Vec::new();
    for manifest in &repo.manifests {
        let document = Fleet::read_manifest(manifest)?;
        for (key, item) in Fleet::dependency_items(&document) {
            let Some(table) = item.as_table_like() else {
                continue;
            };
            let dependency = table
                .get("package")
                .and_then(Item::as_str)
                .unwrap_or(&key)
                .to_string();
            let Some(form) = Fleet::pin_form(table) else {
                continue;
            };
            if fleet.points_at_fleet(repo, manifest, table, &form) {
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

#[cfg(test)]
mod tests {
    use semver::Version;

    use super::pins;
    use crate::testing;

    #[test]
    fn test_usage() {
        let tmp = tempfile::tempdir().unwrap();
        let dependent_dir = tmp.path().join("dependent");
        let dependent = testing::committed_repo(
            &dependent_dir,
            "dependent",
            Version::new(0, 0, 1),
            Some("v0.0.1"),
        );
        let dependency = testing::committed_repo(
            &tmp.path().join("dep"),
            "dep",
            Version::new(0, 0, 2),
            Some("v0.0.2"),
        );
        std::fs::write(
            dependent_dir.join("Cargo.toml"),
            "[package]\nname = \"dependent\"\nversion = \"0.0.1\"\n\n[dependencies]\ndep = { path = \"../dep\" }\n",
        )
        .unwrap();
        let fleet = testing::fleet(tmp.path(), vec![dependent, dependency], Vec::new());
        let repo = fleet
            .repos
            .iter()
            .find(|repo| repo.name == "dependent")
            .unwrap();

        let found = pins(&fleet, repo).unwrap();

        assert_eq!(found.len(), 1);
        assert_eq!(
            found.first().map(|pin| pin.dependency.as_str()),
            Some("dep")
        );
    }
}
