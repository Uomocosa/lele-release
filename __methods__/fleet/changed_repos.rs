use crate::Error;
use crate::fleet::Fleet;

pub fn changed_repos(fleet: &Fleet) -> Result<Vec<(String, u64)>, Error> {
    let mut changed = Vec::new();
    for repo in &fleet.repos {
        match (repo.latest_tag()?, repo.commits_since_tag()?) {
            (None, _) | (Some(_), None) => changed.push((repo.name.clone(), 0)),
            (Some(_), Some(0)) => {}
            (Some(_), Some(count)) => changed.push((repo.name.clone(), count)),
        }
    }
    Ok(changed)
}

#[cfg(test)]
mod tests {
    use semver::Version;

    use super::changed_repos;
    use crate::testing;

    #[test]
    fn test_usage() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = testing::committed_repo(
            &tmp.path().join("probe"),
            "probe",
            Version::new(0, 0, 1),
            None,
        );
        let fleet = testing::fleet(tmp.path(), vec![repo], Vec::new());

        let changed = changed_repos(&fleet).unwrap();

        assert_eq!(changed, vec![(String::from("probe"), 0)]);
    }
}
