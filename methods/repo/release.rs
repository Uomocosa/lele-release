use std::fmt::Write as _;

use semver::Version;
use toml_edit::DocumentMut;

use crate::Error;
use crate::git::Git;
use crate::repo::Repo;

pub fn release(repo: &Repo, next: &Version) -> Result<(), Error> {
    let tag = format!("v{next}");
    write_version(repo, next)?;
    bump_lock_version(repo, next)?;
    stage(repo)?;
    verify_staged_lock(repo, next)?;
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

// needed helper: leave the repo exactly as it was before the attempt
fn rollback(repo: &Repo, tag: &str) {
    let _ = Git::run(&repo.dir, &["reset", "--hard", "origin/main"]);
    let _ = Git::run(&repo.dir, &["tag", "-d", tag]);
}

// needed helper: write the next version into the manifest that owns it
fn write_version(repo: &Repo, next: &Version) -> Result<(), Error> {
    let text = std::fs::read_to_string(&repo.version_manifest).map_err(|error| Error::Io {
        context: format!("{}", repo.version_manifest.display()),
        source: error,
    })?;
    let mut document: DocumentMut = text.parse::<DocumentMut>().map_err(|error| Error::Toml {
        path: repo.version_manifest.clone(),
        source: error,
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
        context: format!("{}", repo.version_manifest.display()),
        source: error,
    })
}

// needed helper: refuse to create a release whose staged lock disagrees with the version
fn verify_staged_lock(repo: &Repo, next: &Version) -> Result<(), Error> {
    if !repo.dir.join("Cargo.lock").exists() {
        return Ok(());
    }
    let staged = Git::output(&repo.dir, &["show", ":Cargo.lock"])?;
    let mut own = false;
    let mut found: Option<String> = None;
    for line in staged.lines() {
        if line == "[[package]]" {
            own = false;
        }
        if let Some(name) = line
            .strip_prefix("name = \"")
            .and_then(|rest| rest.strip_suffix('"'))
        {
            own = repo.packages.iter().any(|package| package.name == name);
        } else if own
            && let Some(version) = line
                .strip_prefix("version = \"")
                .and_then(|rest| rest.strip_suffix('"'))
        {
            found = Some(version.to_string());
            break;
        }
    }
    match found {
        Some(version) if version == next.to_string() => Ok(()),
        Some(version) => Err(Error::Refused(format!(
            "{}: staged lock says {version}, release is {next}",
            repo.name
        ))),
        None => Ok(()),
    }
}

// needed helper: the lock's own package version must move too, or cargo rewrites it later
fn bump_lock_version(repo: &Repo, next: &Version) -> Result<(), Error> {
    let lock = repo.dir.join("Cargo.lock");
    if !lock.exists() {
        return Ok(());
    }
    let text = std::fs::read_to_string(&lock).map_err(|error| Error::Io {
        context: format!("{}", lock.display()),
        source: error,
    })?;
    let mut out = String::new();
    let mut own = false;
    for line in text.lines() {
        if line == "[[package]]" {
            own = false;
        }
        if let Some(name) = line
            .strip_prefix("name = \"")
            .and_then(|rest| rest.strip_suffix('"'))
        {
            own = repo.packages.iter().any(|package| package.name == name);
        } else if own && line.starts_with("version = ") {
            let _ = writeln!(out, "version = \"{next}\"");
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    std::fs::write(&lock, out).map_err(|error| Error::Io {
        context: format!("{}", lock.display()),
        source: error,
    })
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
