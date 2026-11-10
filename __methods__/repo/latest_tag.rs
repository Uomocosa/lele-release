use crate::Error;
use crate::git::Git;
use crate::repo::Repo;

pub fn latest_tag(repo: &Repo) -> Result<Option<String>, Error> {
    let listed = Git::output(
        &repo.dir,
        &["tag", "--list", "v*", "--sort=-version:refname"],
    )?;
    let tag = listed.lines().find(|line| {
        line.trim_start_matches('v')
            .parse::<semver::Version>()
            .is_ok()
    });
    Ok(tag.map(str::to_string))
}

#[cfg(test)]
mod tests {
    use super::latest_tag;

    #[test]
    fn test_usage() {
        let _ = latest_tag;
    }
}
