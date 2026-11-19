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
    use semver::Version;
    use toml_edit::DocumentMut;

    use super::dependency_targets;
    use crate::testing;

    #[test]
    fn test_usage() {
        let tmp = tempfile::tempdir().unwrap();
        let app_dir = tmp.path().join("app");
        let lib_dir = tmp.path().join("lib");
        std::fs::create_dir_all(&app_dir).unwrap();
        std::fs::create_dir_all(&lib_dir).unwrap();
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
        let document: DocumentMut = "[dependencies]\nlib = { path = \"../lib\" }\n"
            .parse()
            .unwrap();

        let targets = dependency_targets(&repos, &app_dir, &document);

        let names: Vec<&str> = targets.iter().map(|repo| repo.name.as_str()).collect();
        assert_eq!(names, vec!["lib"]);
    }
}
