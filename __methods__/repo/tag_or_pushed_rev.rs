use crate::Error;
use crate::git::Git;
use crate::repo::Repo;

pub fn tag_or_pushed_rev(dependency: &Repo) -> Result<(bool, String), Error> {
    if let Some(tag) = dependency.latest_tag()? {
        return Ok((true, tag));
    }
    let head = Git::output(&dependency.dir, &["rev-parse", "HEAD"])?;
    let pushed = Git::output(&dependency.dir, &["rev-parse", "origin/main"]).unwrap_or(head);
    Ok((false, pushed))
}

#[cfg(test)]
mod tests {
    use semver::Version;

    use super::tag_or_pushed_rev;
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

        let tagged = tag_or_pushed_rev(&repo);

        assert_eq!(tagged.unwrap(), (true, String::from("v0.0.2")));
    }
}
