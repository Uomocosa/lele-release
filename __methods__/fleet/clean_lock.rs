use std::collections::HashMap;
use std::path::Path;

use crate::Error;
use crate::fleet::Fleet;
use crate::git::Git;

pub fn clean_lock(fleet: &Fleet, lock: &Path, input: &str) -> Result<String, Error> {
    let Some(current) = fleet.repos.iter().find(|repo| lock.starts_with(&repo.dir)) else {
        return Ok(input.to_string());
    };
    let mut sources: HashMap<String, (String, String)> = HashMap::new();
    for repo in fleet.repos.iter().filter(|repo| repo.name != current.name) {
        let Some(tag) = repo.latest_tag()? else {
            continue;
        };
        let sha = Git::output(&repo.dir, &["rev-parse", &format!("{tag}^{{commit}}")])?;
        let version = tag.trim_start_matches('v').to_string();
        let source = format!("git+{}?tag={tag}#{sha}", repo.url.trim_end_matches(".git"));
        for package in &repo.packages {
            sources.insert(package.name.clone(), (version.clone(), source.clone()));
        }
    }
    Ok(current.clean_lock_text(input, &sources))
}

#[cfg(test)]
mod tests {
    use semver::Version;

    use super::clean_lock;
    use crate::testing;

    #[test]
    fn test_usage() {
        let tmp = tempfile::tempdir().unwrap();
        let dependent = testing::committed_repo(
            &tmp.path().join("dependent"),
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
        let lock = dependent.dir.join("Cargo.lock");
        let input = "version = 4\n\n[[package]]\nname = \"dependent\"\nversion = \"0.0.1\"\n\n[[package]]\nname = \"dep\"\nversion = \"0.0.1\"\nsource = \"path+file:///tmp/dep\"\n";
        let fleet = testing::fleet(tmp.path(), vec![dependent, dependency], Vec::new());

        let cleaned = clean_lock(&fleet, &lock, input).unwrap();

        assert!(cleaned.contains("version = \"0.0.2\""), "{cleaned}");
        assert!(
            cleaned.contains("source = \"git+https://example.com/dep?tag=v0.0.2#"),
            "{cleaned}"
        );
    }
}
