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
    use super::pins;

    #[test]
    fn test_usage() {
        let _ = pins;
    }
}
