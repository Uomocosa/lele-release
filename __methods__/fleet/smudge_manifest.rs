use std::path::Path;

use toml_edit::DocumentMut;

use crate::Error;
use crate::fleet::Fleet;

pub fn smudge_manifest(fleet: &Fleet, manifest: &Path, input: &str) -> Result<String, Error> {
    let mut document: DocumentMut = input.parse::<DocumentMut>().map_err(|error| Error::Toml {
        path: manifest.to_path_buf(),
        source: error,
    })?;
    let base = manifest
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();
    let mut touched = false;
    for section in [
        "dependencies",
        "dev-dependencies",
        "build-dependencies",
        "workspace.dependencies",
    ] {
        touched |= fleet.smudge_manifest_section(&mut document, section, &base);
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

    use super::smudge_manifest;
    use crate::testing;

    #[test]
    fn test_usage() {
        let tmp = tempfile::tempdir().unwrap();
        let dependent_dir = tmp.path().join("dependent");
        let dep_dir = tmp.path().join("dep");
        std::fs::create_dir_all(&dependent_dir).unwrap();
        std::fs::create_dir_all(&dep_dir).unwrap();
        std::fs::write(
            dep_dir.join("Cargo.toml"),
            "[package]\nname = \"dep\"\nversion = \"0.0.2\"\n",
        )
        .unwrap();
        let dependent = testing::repo(
            "dependent",
            &dependent_dir,
            "https://example.com/dependent.git",
            "dependent",
            Version::new(0, 0, 1),
        );
        let dependency = testing::repo(
            "dep",
            &dep_dir,
            "https://example.com/dep.git",
            "dep",
            Version::new(0, 0, 2),
        );
        let manifest = dependent_dir.join("Cargo.toml");
        let input = "[package]\nname = \"dependent\"\nversion = \"0.0.1\"\n\n[dependencies]\ndep = { git = \"https://example.com/dep.git\", tag = \"v0.0.2\" }\n";
        let fleet = testing::fleet(tmp.path(), vec![dependent, dependency], Vec::new());

        let smudged = smudge_manifest(&fleet, &manifest, input).unwrap();

        assert!(smudged.contains("path = \"../dep\""), "{smudged}");
        assert!(!smudged.contains("git ="), "{smudged}");
    }
}
