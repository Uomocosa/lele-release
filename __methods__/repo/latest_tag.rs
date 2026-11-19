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
    use semver::Version;

    use super::latest_tag;
    use crate::testing;

    #[test]
    fn test_usage() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = testing::committed_repo(
            &tmp.path().join("probe"),
            "probe",
            Version::new(0, 0, 2),
            Some("v0.0.2"),
        );

        let tag = latest_tag(&repo);

        assert_eq!(tag.unwrap(), Some(String::from("v0.0.2")));
    }
}
