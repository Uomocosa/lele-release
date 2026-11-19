use std::collections::HashMap;
use std::path::Path;

use crate::fleet::Fleet;

pub fn smudge_lock(fleet: &Fleet, lock: &Path, input: &str) -> String {
    let Some(current) = fleet.repos.iter().find(|repo| lock.starts_with(&repo.dir)) else {
        return input.to_string();
    };
    let mut versions: HashMap<String, String> = HashMap::new();
    for repo in fleet.repos.iter().filter(|repo| repo.name != current.name) {
        for package in &repo.packages {
            versions.insert(package.name.clone(), repo.version.to_string());
        }
    }
    current.smudge_lock_text(input, &versions)
}

#[cfg(test)]
mod tests {
    use semver::Version;

    use super::smudge_lock;
    use crate::testing;

    #[test]
    fn test_usage() {
        let tmp = tempfile::tempdir().unwrap();
        let dependent_dir = tmp.path().join("dependent");
        let dep_dir = tmp.path().join("dep");
        std::fs::create_dir_all(&dependent_dir).unwrap();
        std::fs::create_dir_all(&dep_dir).unwrap();
        let dependent = testing::repo(
            "dependent",
            &dependent_dir,
            "https://example.com/dependent.git",
            "dependent",
            Version::new(0, 0, 1),
        );
        let dependency = testing::repo(
            "dep",
            &dep_dir,
            "https://example.com/dep.git",
            "dep",
            Version::new(0, 0, 2),
        );
        let lock = dependent_dir.join("Cargo.lock");
        let input = "version = 4\n\n[[package]]\nname = \"dependent\"\nversion = \"0.0.1\"\n\n[[package]]\nname = \"dep\"\nversion = \"0.0.2\"\nsource = \"git+https://example.com/dep?tag=v0.0.2#abc\"\n";
        let fleet = testing::fleet(tmp.path(), vec![dependent, dependency], Vec::new());

        let smudged = smudge_lock(&fleet, &lock, input);

        assert!(
            smudged.contains("name = \"dep\"\nversion = \"0.0.2\""),
            "{smudged}"
        );
        assert!(!smudged.contains("source ="), "{smudged}");
    }
}
