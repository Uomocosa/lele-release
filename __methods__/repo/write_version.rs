use semver::Version;
use toml_edit::DocumentMut;

use crate::Error;
use crate::repo::Repo;

pub fn write_version(repo: &Repo, next: &Version) -> Result<(), Error> {
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

#[cfg(test)]
mod tests {
    use semver::Version;

    use super::write_version;
    use crate::testing;

    const MANIFEST: &str = "[package]\nname = \"probe\"\nversion = \"0.0.1\"\n";

    #[test]
    fn test_usage() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("Cargo.toml"), MANIFEST).unwrap();
        let repo = testing::repo(
            "probe",
            tmp.path(),
            "https://example.com/probe.git",
            "probe",
            Version::new(0, 0, 1),
        );

        let result = write_version(&repo, &Version::new(0, 0, 2));

        assert!(result.is_ok());
        let written = std::fs::read_to_string(tmp.path().join("Cargo.toml")).unwrap();
        assert!(written.contains("version = \"0.0.2\""), "{written}");
    }
}
