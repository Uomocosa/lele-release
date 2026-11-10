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
    use super::has_only_fleet_drift;

    #[test]
    fn test_usage() {
        let _ = has_only_fleet_drift;
    }
}
