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
    Git::run(
        &repo.dir,
        &["commit", "-m", &format!("chore(release): {next}")],
    )?;
    Git::run(
        &repo.dir,
        &["tag", "-a", &tag, "-m", &format!("{} {next}", repo.name)],
    )?;
    if let Err(error) = Git::run(&repo.dir, &["push", "--atomic", "origin", "main", &tag]) {
        let _ = Git::run(&repo.dir, &["reset", "--hard", "origin/main"]);
        let _ = Git::run(&repo.dir, &["tag", "-d", &tag]);
        return Err(error);
    }
    Ok(())
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
    let Ok(relative) = repo.version_manifest.strip_prefix(&repo.dir) else {
        return Err(Error::Refused(format!(
            "{}: version manifest is outside the repo",
            repo.name
        )));
    };
    let version_path = relative.display().to_string();
    let mut args = vec!["add", "--", version_path.as_str()];
    if repo.dir.join("Cargo.lock").exists() {
        args.push("Cargo.lock");
    }
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
