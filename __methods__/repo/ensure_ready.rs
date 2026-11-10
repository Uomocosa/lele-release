use crate::Error;
use crate::fleet::Fleet;
use crate::git::Git;
use crate::repo::Repo;

pub fn ensure_ready(repo: &Repo, fleet: &Fleet) -> Result<(), Error> {
    let _ = Git::output(&repo.dir, &["fetch", "--quiet", "origin"]);
    repo.run_just_gate()?;
    repo.refuse_unclean_tree(fleet)?;
    let branch = Git::output(&repo.dir, &["rev-parse", "--abbrev-ref", "HEAD"])?;
    if branch != "main" {
        return Err(Error::Refused(format!(
            "{}: on branch `{branch}`, expected `main`",
            repo.name
        )));
    }
    let counts = Git::output(
        &repo.dir,
        &["rev-list", "--left-right", "--count", "origin/main...HEAD"],
    )?;
    let mut numbers = counts.split_whitespace();
    let behind = numbers.next().unwrap_or("?");
    let ahead = numbers.next().unwrap_or("?");
    if behind != "0" || ahead != "0" {
        return Err(Error::Refused(format!(
            "{}: not in sync with origin/main (behind {behind}, ahead {ahead})",
            repo.name
        )));
    }
    repo.latest_tag()?.map_or(Ok(()), |tag| {
        let tagged = tag.trim_start_matches('v');
        if tagged == repo.version.to_string() {
            Ok(())
        } else {
            Err(Error::Refused(format!(
                "{}: manifest version {} does not match tag {tag}",
                repo.name, repo.version
            )))
        }
    })
}

#[cfg(test)]
mod tests {
    use super::ensure_ready;

    #[test]
    fn test_usage() {
        let _ = ensure_ready;
    }
}
