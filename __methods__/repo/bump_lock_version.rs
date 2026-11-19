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
    use semver::Version;

    use super::bump_lock_version;
    use crate::testing;

    const LOCK: &str = "version = 4\n\n[[package]]\nname = \"probe\"\nversion = \"0.0.1\"\n\n[[package]]\nname = \"other\"\nversion = \"9.9.9\"\n";

    #[test]
    fn test_usage() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("Cargo.lock"), LOCK).unwrap();
        let repo = testing::repo(
            "probe",
            tmp.path(),
            "https://example.com/probe.git",
            "probe",
            Version::new(0, 0, 1),
        );

        let result = bump_lock_version(&repo, &Version::new(0, 0, 2));

        assert!(result.is_ok());
        let bumped = std::fs::read_to_string(tmp.path().join("Cargo.lock")).unwrap();
        assert!(
            bumped.contains("name = \"probe\"\nversion = \"0.0.2\""),
            "{bumped}"
        );
        assert!(
            bumped.contains("name = \"other\"\nversion = \"9.9.9\""),
            "{bumped}"
        );
    }
}
