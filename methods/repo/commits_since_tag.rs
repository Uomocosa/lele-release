use crate::Error;
use crate::git::Git;
use crate::repo::Repo;

pub fn commits_since_tag(repo: &Repo) -> Result<Option<u64>, Error> {
    let Some(tag) = repo.latest_tag()? else {
        return Ok(None);
    };
    let range = format!("{tag}..HEAD");
    let counted = Git::output(&repo.dir, &["rev-list", "--count", &range])?;
    let count = counted.parse::<u64>().map_err(|error| {
        Error::Refused(format!(
            "{}: cannot read commit count `{counted}`: {error}",
            repo.name
        ))
    })?;
    Ok(Some(count))
}

#[cfg(test)]
mod tests {
    use super::commits_since_tag;

    #[test]
    fn test_usage() {
        let _ = commits_since_tag;
    }
}
