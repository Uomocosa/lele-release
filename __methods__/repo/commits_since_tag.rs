use crate::Error;
use crate::git::Git;
use crate::repo::Repo;

pub fn commits_since_tag(repo: &Repo) -> Result<Option<u64>, Error> {
    let Some(tag) = repo.latest_tag()? else {
        return Ok(None);
    };
    let range = format!("{tag}..HEAD");
    let counted = Git::output(&repo.dir, &["rev-list", "--count", &range])?;
    let count = counted
        .parse::<u64>()
        .map_err(|source| Error::CommitCount {
            context: format!("{}: cannot read commit count `{counted}`", repo.name),
            source,
        })?;
    Ok(Some(count))
}

#[cfg(test)]
mod tests {
    use semver::Version;

    use super::commits_since_tag;
    use crate::Git;
    use crate::testing;

    #[test]
    fn test_usage() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("probe");
        let repo = testing::committed_repo(&dir, "probe", Version::new(0, 0, 2), Some("v0.0.2"));
        std::fs::write(dir.join("note.txt"), "one\n").unwrap();
        Git::run(&dir, &["add", "--", "note.txt"]).unwrap();
        Git::run(
            &dir,
            &[
                "-c",
                "user.name=t",
                "-c",
                "user.email=t@t",
                "commit",
                "-qm",
                "note",
            ],
        )
        .unwrap();

        let count = commits_since_tag(&repo);

        assert_eq!(count.unwrap(), Some(1));
    }
}
