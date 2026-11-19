use semver::Version;
use toml_edit::DocumentMut;

#[must_use]
pub fn workspace_version(document: &DocumentMut) -> Option<Version> {
    let version = document
        .get("workspace")?
        .as_table_like()?
        .get("package")?
        .as_table_like()?
        .get("version")?
        .as_str()?;
    version.parse::<Version>().ok()
}

#[cfg(test)]
mod tests {
    use semver::Version;
    use toml_edit::DocumentMut;

    use super::workspace_version;

    const WORKSPACE: &str = "[workspace.package]\nversion = \"0.4.2\"\n";

    #[test]
    fn test_usage() {
        let document: DocumentMut = WORKSPACE.parse().unwrap();

        let version = workspace_version(&document);

        assert_eq!(version, Some(Version::new(0, 4, 2)));
    }
}
