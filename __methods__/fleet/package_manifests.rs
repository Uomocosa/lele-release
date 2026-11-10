use std::path::{Path, PathBuf};

use crate::Error;

pub fn package_manifests(dir: &Path) -> Result<Vec<PathBuf>, Error> {
    let root = dir.join("Cargo.toml");
    if !root.exists() {
        return Ok(Vec::new());
    }
    let output = std::process::Command::new("cargo")
        .args([
            "metadata",
            "--no-deps",
            "--format-version",
            "1",
            "--offline",
            "--manifest-path",
        ])
        .arg(&root)
        .output()
        .map_err(|error| Error::Io {
            context: format!("cannot run cargo metadata in {}", dir.display()),
            source: error,
        })?;
    if !output.status.success() {
        return Err(Error::Refused(format!(
            "{}: cargo metadata failed\n{}",
            dir.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    let metadata: serde_json::Value = serde_json::from_slice(&output.stdout).map_err(|_| {
        Error::Refused(format!(
            "{}: cargo metadata output is not JSON",
            dir.display()
        ))
    })?;
    let mut discovered: Vec<String> = Vec::new();
    if let Some(packages) = metadata
        .get("packages")
        .and_then(serde_json::Value::as_array)
    {
        for package in packages {
            let Some(path) = package
                .get("manifest_path")
                .and_then(serde_json::Value::as_str)
            else {
                continue;
            };
            if !discovered.iter().any(|known| known == path) {
                discovered.push(path.to_string());
            }
        }
    }
    let canonical = dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());
    let mut out = vec![root];
    for absolute in discovered {
        let anchored = Path::new(&absolute).strip_prefix(&canonical).map_or_else(
            |_| PathBuf::from(absolute.clone()),
            |relative| dir.join(relative),
        );
        if !out.contains(&anchored) {
            out.push(anchored);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::package_manifests;

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(
            package_manifests(dir.path()).unwrap(),
            Vec::<PathBuf>::new()
        );
        std::fs::write(dir.path().join("Cargo.toml"), "not toml [[[[").unwrap();
        assert!(package_manifests(dir.path()).is_err());
    }
}
