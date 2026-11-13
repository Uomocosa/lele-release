use crate::Error;
use crate::fleet::Fleet;
use crate::repo::Repo;

pub fn repo_named<'a>(fleet: &'a Fleet, name: &str) -> Result<&'a Repo, Error> {
    fleet
        .repos
        .iter()
        .find(|repo| repo.name == name)
        .ok_or_else(|| Error::Refused(format!("unknown repo `{name}`")))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::repo_named;
    use crate::Error;
    use crate::fleet::Fleet;

    #[test]
    fn test_usage() {
        let fleet = Fleet {
            root: PathBuf::new(),
            repos: Vec::new(),
            dependencies: Vec::new(),
        };
        assert!(matches!(
            repo_named(&fleet, "missing"),
            Err(Error::Refused(message)) if message == "unknown repo `missing`"
        ));
    }
}
