use std::fmt::Write as _;

use semver::Version;

use crate::Error;
use crate::repo::Repo;

pub fn bump_lock_version(repo: &Repo, next: &Version) -> Result<(), Error> {
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

#[cfg(test)]
mod tests {
    use super::bump_lock_version;

    #[test]
    fn test_usage() {
        let _ = bump_lock_version;
    }
}
