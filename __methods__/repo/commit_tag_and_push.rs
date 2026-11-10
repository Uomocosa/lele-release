use semver::Version;

use crate::Error;
use crate::git::Git;
use crate::repo::Repo;

pub fn commit_tag_and_push(repo: &Repo, next: &Version, tag: &str) -> Result<(), Error> {
    Git::run(
        &repo.dir,
        &[
            "commit",
            "--no-verify",
            "-m",
            &format!("chore(release): {next}"),
        ],
    )?;
    Git::run(
        &repo.dir,
        &["tag", "-a", tag, "-m", &format!("{} {next}", repo.name)],
    )?;
    Git::run(
        &repo.dir,
        &["push", "--no-verify", "--atomic", "origin", "main", tag],
    )
}

#[cfg(test)]
mod tests {
    use super::commit_tag_and_push;

    #[test]
    fn test_usage() {
        let _ = commit_tag_and_push;
    }
}
