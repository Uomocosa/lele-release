use crate::Error;
use crate::basic;
use crate::fleet::Fleet;

pub fn refuse_dirty_after_release(
    fleet: &Fleet,
    completed: &[basic::structs::ReleaseStep],
) -> Result<(), Error> {
    let mut dirty: Vec<String> = Vec::new();
    for step in completed {
        let repo = fleet.repo_named(&step.repo)?;
        let paths = repo.untrimmed_status_paths()?;
        if !paths.is_empty() {
            dirty.push(format!("{}: {}", repo.name, paths.join(", ")));
        }
    }
    if dirty.is_empty() {
        Ok(())
    } else {
        Err(Error::Refused(format!(
            "working tree is not clean after release: {}",
            dirty.join("; ")
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::refuse_dirty_after_release;

    #[test]
    fn test_usage() {
        let _ = refuse_dirty_after_release;
    }
}
