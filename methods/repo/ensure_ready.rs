use crate::Error;
use crate::git::Git;
use crate::repo::Repo;

pub fn ensure_ready(repo: &Repo) -> Result<(), Error> {
    let _ = Git::output(&repo.dir, &["fetch", "--quiet", "origin"]);
    gate(repo)?;
    let status = Git::output(&repo.dir, &["status", "--porcelain"])?;
    let lock_only = !status.is_empty() && status.lines().all(|line| line.ends_with("Cargo.lock"));
    if lock_only {
        let _ = Git::run(&repo.dir, &["checkout", "--", "Cargo.lock"]);
    }
    let status = Git::output(&repo.dir, &["status", "--porcelain"])?;
    if !status.is_empty() {
        return Err(Error::Refused(format!(
            "{}: working tree is not clean:\n{status}",
            repo.name
        )));
    }
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

// needed helper: the real gate runs before the release, so the metadata commit can skip hooks
fn gate(repo: &Repo) -> Result<(), Error> {
    for recipe in ["fmt", "lint", "test"] {
        let output = std::process::Command::new("just")
            .arg(recipe)
            .current_dir(&repo.dir)
            .output()
            .map_err(|error| Error::Io {
                message: format!("{}: cannot run `just {recipe}`: {error}", repo.name),
            })?;
        if !output.status.success() {
            return Err(Error::Refused(format!(
                "{}: `just {recipe}` failed\n{}",
                repo.name,
                String::from_utf8_lossy(&output.stderr)
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::ensure_ready;

    #[test]
    fn test_usage() {
        let _ = ensure_ready;
    }
}
