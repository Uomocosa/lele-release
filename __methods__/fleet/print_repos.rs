use crate::Error;
use crate::fleet::Fleet;

pub fn print_repos(fleet: &Fleet) -> Result<(), Error> {
    let width = fleet
        .repos
        .iter()
        .map(|repo| repo.name.len())
        .max()
        .unwrap_or(0)
        .max(12);
    println!(
        "{:<width$}  {:>7}  {:<8}  {:>5}  tree",
        "repo", "version", "tag", "since"
    );
    for repo in &fleet.repos {
        let tag = repo.latest_tag()?.unwrap_or_else(|| String::from("-"));
        let since = repo
            .commits_since_tag()?
            .map_or_else(|| String::from("-"), |count| count.to_string());
        println!(
            "{:<width$}  {:>7}  {:<8}  {:>5}  {}",
            repo.name,
            repo.version,
            tag,
            since,
            repo.tree_state()
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use semver::Version;

    use super::print_repos;
    use crate::testing;

    #[test]
    fn test_usage() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = testing::committed_repo(
            &tmp.path().join("probe"),
            "probe",
            Version::new(0, 0, 2),
            Some("v0.0.2"),
        );
        let fleet = testing::fleet(tmp.path(), vec![repo], Vec::new());

        let result = print_repos(&fleet);

        assert!(result.is_ok());
    }
}
