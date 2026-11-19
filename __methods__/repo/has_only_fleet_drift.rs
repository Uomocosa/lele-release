use crate::Error;
use crate::fleet::Fleet;
use crate::git::Git;
use crate::repo::Repo;

pub fn has_only_fleet_drift(repo: &Repo, fleet: &Fleet, relative: &str) -> Result<bool, Error> {
    let path = repo.dir.join(relative);
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Ok(false);
    };
    let cleaned = fleet.clean_manifest(&path, &text)?;
    let index = Git::output(&repo.dir, &["show", &format!(":{relative}")])?;
    Ok(fleet.without_fleet_lines(&cleaned) == fleet.without_fleet_lines(&index))
}

#[cfg(test)]
mod tests {
    use semver::Version;

    use super::has_only_fleet_drift;
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

        let committed = has_only_fleet_drift(repo, &fleet, "Cargo.toml");
        let missing = has_only_fleet_drift(repo, &fleet, "absent.toml");

        assert!(committed.unwrap());
        assert!(!missing.unwrap());
    }
}
