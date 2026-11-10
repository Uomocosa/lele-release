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
    use super::changed_repos;

    #[test]
    fn test_usage() {
        let _ = changed_repos;
    }
}
