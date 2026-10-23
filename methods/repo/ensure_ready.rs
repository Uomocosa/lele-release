use crate::Error;
use crate::fleet::Fleet;
use crate::git::Git;
use crate::repo::Repo;

pub fn ensure_ready(repo: &Repo, fleet: &Fleet) -> Result<(), Error> {
    let _ = Git::output(&repo.dir, &["fetch", "--quiet", "origin"]);
    gate(repo)?;
    settle(repo, fleet)?;
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
                context: format!("{}: cannot run `just {recipe}`", repo.name),
                source: error,
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

// needed helper: a lock is generated, and a stale pin makes it differ until the release re-pins
fn settle(repo: &Repo, fleet: &Fleet) -> Result<(), Error> {
    for path in status_paths(repo)? {
        let path = path.as_str();
        if path.ends_with("Cargo.lock") {
            continue;
        }
        if path.ends_with("Cargo.toml") && manifest_drift_only(repo, fleet, path)? {
            continue;
        }
        return Err(Error::Refused(format!(
            "{}: working tree is not clean: {path}",
            repo.name
        )));
    }
    Ok(())
}

// needed helper: raw porcelain paths (Git::output trims, which shifts the XY prefix)
fn status_paths(repo: &Repo) -> Result<Vec<String>, Error> {
    let output = std::process::Command::new("git")
        .args(["status", "--porcelain", "-z"])
        .current_dir(&repo.dir)
        .output()
        .map_err(|error| Error::Io {
            context: format!("{}: cannot read git status", repo.name),
            source: error,
        })?;
    let text = String::from_utf8_lossy(&output.stdout);
    Ok(text
        .split('\0')
        .filter_map(|entry| entry.get(3..))
        .map(str::to_string)
        .collect())
}

// needed helper: true when the only difference in a manifest is fleet dependency entries
fn manifest_drift_only(repo: &Repo, fleet: &Fleet, relative: &str) -> Result<bool, Error> {
    let path = repo.dir.join(relative);
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Ok(false);
    };
    let cleaned = fleet.clean_manifest(&path, &text)?;
    let index = Git::output(&repo.dir, &["show", &format!(":{relative}")])?;
    Ok(without_fleet_lines(&cleaned, fleet) == without_fleet_lines(&index, fleet))
}

// needed helper: drop inline dependency entries that name a fleet package
fn without_fleet_lines(text: &str, fleet: &Fleet) -> String {
    let names: Vec<String> = fleet
        .repos
        .iter()
        .flat_map(|repo| repo.packages.iter().map(|package| package.name.clone()))
        .collect();
    text.lines()
        .filter(|line| {
            let trimmed = line.trim_start();
            !names
                .iter()
                .any(|name| trimmed.starts_with(&format!("{name} = ")))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::ensure_ready;

    #[test]
    fn test_usage() {
        let _ = ensure_ready;
    }
}
