use crate::Error;
use crate::fleet::Fleet;
use crate::repo::Repo;

pub fn refuse_unclean_tree(repo: &Repo, fleet: &Fleet) -> Result<(), Error> {
    for path in repo.untrimmed_status_paths()? {
        let path = path.as_str();
        if path.ends_with("Cargo.lock") {
            continue;
        }
        if path.ends_with("Cargo.toml") && repo.has_only_fleet_drift(fleet, path)? {
            continue;
        }
        return Err(Error::Refused(format!(
            "{}: working tree is not clean: {path}",
            repo.name
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use semver::Version;

    use super::refuse_unclean_tree;
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
        let repo = fleet.repos.first().unwrap();

        let clean = refuse_unclean_tree(repo, &fleet);
        std::fs::write(repo.dir.join("dirty.txt"), "x").unwrap();
        let dirty = refuse_unclean_tree(repo, &fleet);

        assert!(clean.is_ok());
        assert!(dirty.is_err());
    }
}
