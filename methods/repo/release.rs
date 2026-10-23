use semver::Version;
use toml_edit::DocumentMut;

use crate::Error;
use crate::fleet::Fleet;
use crate::git::Git;
use crate::repo::Repo;

pub fn release(repo: &Repo, next: &Version, fleet: &Fleet) -> Result<(), Error> {
    let tag = format!("v{next}");
    write_version(repo, next)?;
    normalise_lock(repo, fleet)?;
    stage(repo)?;
    match publish(repo, next, &tag) {
        Ok(()) => Ok(()),
        Err(error) => {
            rollback(repo, &tag);
            Err(error)
        }
    }
}

// needed helper: commit, tag, and push atomically
fn publish(repo: &Repo, next: &Version, tag: &str) -> Result<(), Error> {
    Git::run(
        &repo.dir,
        &["commit", "-m", &format!("chore(release): {next}")],
    )?;
    Git::run(
        &repo.dir,
        &["tag", "-a", tag, "-m", &format!("{} {next}", repo.name)],
    )?;
    Git::run(&repo.dir, &["push", "--atomic", "origin", "main", tag])
}

// needed helper: leave the repo exactly as it was before the attempt
fn rollback(repo: &Repo, tag: &str) {
    let _ = Git::run(&repo.dir, &["reset", "--hard", "origin/main"]);
    let _ = Git::run(&repo.dir, &["tag", "-d", tag]);
}

// needed helper: write the next version into the manifest that owns it
fn write_version(repo: &Repo, next: &Version) -> Result<(), Error> {
    let text = std::fs::read_to_string(&repo.version_manifest).map_err(|error| Error::Io {
        message: format!("{}: {error}", repo.version_manifest.display()),
    })?;
    let mut document: DocumentMut = text.parse::<DocumentMut>().map_err(|error| Error::Toml {
        path: repo.version_manifest.display().to_string(),
        message: error.to_string(),
    })?;
    let inherited = document
        .get("workspace")
        .and_then(toml_edit::Item::as_table_like)
        .and_then(|workspace| workspace.get("package"))
        .and_then(toml_edit::Item::as_table_like)
        .and_then(|package| package.get("version"))
        .is_some();
    let written = if inherited {
        document
            .get_mut("workspace")
            .and_then(toml_edit::Item::as_table_like_mut)
            .and_then(|workspace| workspace.get_mut("package"))
            .and_then(toml_edit::Item::as_table_like_mut)
            .and_then(|package| package.get_mut("version"))
    } else {
        document
            .get_mut("package")
            .and_then(toml_edit::Item::as_table_like_mut)
            .and_then(|package| package.get_mut("version"))
    };
    let Some(field) = written else {
        return Err(Error::Refused(format!(
            "{}: no version field in {}",
            repo.name,
            repo.version_manifest.display()
        )));
    };
    *field = toml_edit::value(next.to_string());
    std::fs::write(&repo.version_manifest, document.to_string()).map_err(|error| Error::Io {
        message: format!("{}: {error}", repo.version_manifest.display()),
    })
}

// needed helper: rewrite fleet entries in Cargo.lock to the released git source
fn normalise_lock(repo: &Repo, fleet: &Fleet) -> Result<(), Error> {
    let lock = repo.dir.join("Cargo.lock");
    if !lock.exists() {
        return Ok(());
    }
    let text = std::fs::read_to_string(&lock).map_err(|error| Error::Io {
        message: format!("{}: {error}", lock.display()),
    })?;
    let mut document: DocumentMut = text.parse::<DocumentMut>().map_err(|error| Error::Toml {
        path: lock.display().to_string(),
        message: error.to_string(),
    })?;
    let Some(packages) = document
        .get_mut("package")
        .and_then(toml_edit::Item::as_array_of_tables_mut)
    else {
        return Ok(());
    };
    let mut changed = false;
    for table in packages.iter_mut() {
        let Some(name) = table
            .get("name")
            .and_then(toml_edit::Item::as_str)
            .map(str::to_string)
        else {
            continue;
        };
        if repo.packages.iter().any(|package| package.name == name) {
            continue;
        }
        let Some(dependency) = fleet.repos.iter().find(|candidate| {
            candidate
                .packages
                .iter()
                .any(|package| package.name == name)
        }) else {
            continue;
        };
        let is_edge = fleet
            .dependencies
            .iter()
            .any(|edge| edge.dependent == repo.name && edge.dependency == dependency.name);
        if !is_edge {
            continue;
        }
        let source = git_source(dependency)?;
        table.insert("source", toml_edit::value(source));
        table.insert("version", toml_edit::value(dependency.version.to_string()));
        changed = true;
    }
    if changed {
        std::fs::write(&lock, document.to_string()).map_err(|error| Error::Io {
            message: format!("{}: {error}", lock.display()),
        })?;
    }
    Ok(())
}

// needed helper: the lock `source` value for a freshly released dependency
fn git_source(dependency: &Repo) -> Result<String, Error> {
    let url = dependency.url.trim_end_matches(".git").to_string();
    let head = Git::output(&dependency.dir, &["rev-parse", "HEAD"])?;
    match (dependency.latest_tag()?, dependency.commits_since_tag()?) {
        (Some(tag), Some(0)) => Ok(format!("git+{url}?tag={tag}#{head}")),
        _ => Ok(format!("git+{url}?rev={head}#{head}")),
    }
}

// needed helper: stage only the version file and the lock
fn stage(repo: &Repo) -> Result<(), Error> {
    let mut paths: Vec<String> = Vec::new();
    for manifest in &repo.manifests {
        let Ok(relative) = manifest.strip_prefix(&repo.dir) else {
            return Err(Error::Refused(format!(
                "{}: manifest is outside the repo",
                repo.name
            )));
        };
        paths.push(relative.display().to_string());
    }
    if repo.dir.join("Cargo.lock").exists() {
        paths.push(String::from("Cargo.lock"));
    }
    let mut args = vec!["add", "--"];
    args.extend(paths.iter().map(String::as_str));
    Git::run(&repo.dir, &args)
}

#[cfg(test)]
mod tests {
    use super::release;

    #[test]
    fn test_usage() {
        let _ = release;
    }
}
