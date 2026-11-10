use std::path::Path;

use toml_edit::DocumentMut;

use crate::fleet::Fleet;
use crate::repo::Repo;

pub fn dependency_targets<'a>(
    repos: &'a [Repo],
    base: &Path,
    document: &DocumentMut,
) -> Vec<&'a Repo> {
    let mut targets = Vec::new();
    for section in ["dependencies", "dev-dependencies", "build-dependencies"] {
        if let Some(table) = document
            .get(section)
            .and_then(toml_edit::Item::as_table_like)
        {
            Fleet::collect_targets(repos, base, table, &mut targets);
        }
    }
    if let Some(table) = document
        .get("workspace")
        .and_then(toml_edit::Item::as_table_like)
        .and_then(|workspace| workspace.get("dependencies"))
        .and_then(toml_edit::Item::as_table_like)
    {
        Fleet::collect_targets(repos, base, table, &mut targets);
    }
    targets
}

#[cfg(test)]
mod tests {
    use super::dependency_targets;

    #[test]
    fn test_usage() {
        let _ = dependency_targets;
    }
}
