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
    use super::refuse_unclean_tree;

    #[test]
    fn test_usage() {
        let _ = refuse_unclean_tree;
    }
}
