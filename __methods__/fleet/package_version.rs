use std::path::PathBuf;

use semver::Version;

use crate::Error;
use crate::basic;
use crate::fleet::Fleet;

pub fn package_version(
    packages: &[basic::structs::Package],
    repo: &str,
) -> Result<(PathBuf, Version), Error> {
    let Some(first) = packages.first() else {
        return Err(Error::Refused(format!("{repo}: no Cargo package found")));
    };
    let mut found: Option<Version> = None;
    for package in packages {
        let document = Fleet::read_manifest(&package.manifest)?;
        let version = document
            .get("package")
            .and_then(toml_edit::Item::as_table_like)
            .and_then(|table| table.get("version"))
            .and_then(toml_edit::Item::as_str)
            .ok_or_else(|| {
                Error::Refused(format!(
                    "{repo}: package `{}` has no literal version",
                    package.name
                ))
            })?;
        let parsed = version.parse::<Version>().map_err(|source| Error::Semver {
            context: format!(
                "{repo}: package `{}` version `{version}` is not semver",
                package.name
            ),
            source,
        })?;
        match &found {
            None => found = Some(parsed),
            Some(known) if *known == parsed => {}
            Some(known) => {
                return Err(Error::Refused(format!(
                    "{repo}: packages disagree on version ({known} vs {parsed}); the fleet keeps one version per repo"
                )));
            }
        }
    }
    let version = found.unwrap_or_else(|| Version::new(0, 0, 0));
    Ok((first.manifest.clone(), version))
}

#[cfg(test)]
mod tests {
    use semver::Version;

    use super::package_version;
    use crate::Package;

    #[test]
    fn test_usage() {
        let tmp = tempfile::tempdir().unwrap();
        let manifest = tmp.path().join("Cargo.toml");
        std::fs::write(
            &manifest,
            "[package]\nname = \"probe\"\nversion = \"0.0.3\"\n",
        )
        .unwrap();
        let packages = vec![Package {
            name: String::from("probe"),
            manifest: manifest.clone(),
        }];

        let (found, version) = package_version(&packages, "probe").unwrap();

        assert_eq!(found, manifest);
        assert_eq!(version, Version::new(0, 0, 3));
    }
}
