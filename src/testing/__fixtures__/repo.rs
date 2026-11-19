use std::path::Path;

use semver::Version;

use crate::Package;
use crate::Repo;

pub fn repo(name: &str, dir: &Path, url: &str, package: &str, version: Version) -> Repo {
    let manifest = dir.join("Cargo.toml");
    Repo {
        name: name.to_string(),
        dir: dir.to_path_buf(),
        url: url.to_string(),
        packages: vec![Package {
            name: package.to_string(),
            manifest: manifest.clone(),
        }],
        manifests: vec![manifest.clone()],
        version_manifest: manifest,
        version,
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use semver::Version;

    use super::repo;

    #[test]
    fn test_usage() {
        let built = repo(
            "probe",
            Path::new("/probe"),
            "https://example.com/probe.git",
            "probe",
            Version::new(0, 0, 2),
        );
        assert_eq!(built.name, "probe");
        assert_eq!(built.url, "https://example.com/probe.git");
        assert_eq!(built.version, Version::new(0, 0, 2));
    }
}
