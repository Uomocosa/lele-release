use std::path::Path;

use toml_edit::{Item, TableLike};

use crate::fleet::Fleet;
use crate::repo::Repo;

pub fn collect_targets<'a>(
    repos: &'a [Repo],
    base: &Path,
    table: &dyn TableLike,
    targets: &mut Vec<&'a Repo>,
) {
    for (_, item) in table.iter() {
        let Some(entry) = item.as_table_like() else {
            continue;
        };
        let target = entry
            .get("git")
            .and_then(Item::as_str)
            .and_then(|url| repos.iter().find(|repo| repo.matches_url(url)))
            .or_else(|| {
                entry
                    .get("path")
                    .and_then(Item::as_str)
                    .and_then(|path| Fleet::resolve_repo(repos, base, path))
            });
        if let Some(repo) = target {
            targets.push(repo);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use toml_edit::DocumentMut;

    use super::collect_targets;
    use crate::repo::Repo;

    #[test]
    fn test_usage() {
        let repos: Vec<Repo> = Vec::new();
        let document: DocumentMut =
            "[dependencies]\nlost = { git = \"https://example.com/lost.git\" }\nplain = \"1\"\n"
                .parse()
                .unwrap();
        let table = document
            .get("dependencies")
            .and_then(|item| item.as_table_like())
            .unwrap();
        let mut targets = Vec::new();
        collect_targets(&repos, Path::new("."), table, &mut targets);
        assert!(targets.is_empty());
    }
}
