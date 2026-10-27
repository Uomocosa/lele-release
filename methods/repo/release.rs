use semver::Version;

use crate::Error;
use crate::repo::Repo;

pub fn release(repo: &Repo, next: &Version) -> Result<(), Error> {
    let tag = format!("v{next}");
    repo.write_version(next)?;
    repo.bump_lock_version(next)?;
    repo.stage_version_files()?;
    repo.verify_staged_lock(next)?;
    match repo.commit_tag_and_push(next, &tag) {
        Ok(()) => Ok(()),
        Err(error) => {
            repo.rollback_to_origin(&tag);
            Err(error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::release;

    #[test]
    fn test_usage() {
        let _ = release;
    }
}
