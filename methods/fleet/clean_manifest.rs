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
    use super::clean_manifest;

    #[test]
    fn test_usage() {
        let _ = clean_manifest;
    }
}
