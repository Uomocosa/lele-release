use std::path::Path;

use toml_edit::DocumentMut;

use crate::Error;
use crate::fleet::Fleet;

pub fn clean_manifest(fleet: &Fleet, manifest: &Path, input: &str) -> Result<String, Error> {
    let mut document: DocumentMut = input.parse::<DocumentMut>().map_err(|error| Error::Toml {
        path: manifest.to_path_buf(),
        source: error,
    })?;
    let base = manifest
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();
    let current = fleet
        .repos
        .iter()
        .find(|repo| manifest.starts_with(&repo.dir))
        .map(|repo| repo.name.clone())
        .unwrap_or_default();
    let mut touched = false;
    for section in [
        "dependencies",
        "dev-dependencies",
        "build-dependencies",
        "workspace.dependencies",
    ] {
        touched |= fleet.clean_manifest_section(&mut document, section, &base, &current)?;
    }
    Ok(if touched {
        document.to_string()
    } else {
        input.to_string()
    })
}

#[cfg(test)]
mod tests {
    use semver::Version;

    use super::clean_manifest;
    use crate::testing;

    #[test]
    fn test_usage() {
        let tmp = tempfile::tempdir().unwrap();
        let dependent = testing::committed_repo(
            &tmp.path().join("dependent"),
            "dependent",
            Version::new(0, 0, 1),
            Some("v0.0.1"),
        );
        let dependency = testing::committed_repo(
            &tmp.path().join("dep"),
            "dep",
            Version::new(0, 0, 2),
            Some("v0.0.2"),
        );
        let manifest = dependent.dir.join("Cargo.toml");
        let input = "[package]\nname = \"dependent\"\nversion = \"0.0.1\"\n\n[dependencies]\ndep = { path = \"../dep\" }\n";
        std::fs::write(&manifest, input).unwrap();
        let fleet = testing::fleet(tmp.path(), vec![dependent, dependency], Vec::new());

        let cleaned = clean_manifest(&fleet, &manifest, input).unwrap();

        assert!(
            cleaned.contains("git = \"https://example.com/dep\""),
            "{cleaned}"
        );
        assert!(cleaned.contains("tag = \"v0.0.2\""), "{cleaned}");
        assert!(!cleaned.contains("path ="), "{cleaned}");
    }
}
