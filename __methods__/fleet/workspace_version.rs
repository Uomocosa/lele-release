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
    use super::workspace_version;

    #[test]
    fn test_usage() {
        let _ = workspace_version;
    }
}
