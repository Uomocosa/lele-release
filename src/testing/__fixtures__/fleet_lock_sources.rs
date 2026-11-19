use std::collections::HashMap;

pub fn fleet_lock_sources() -> HashMap<String, (String, String)> {
    const GIT: &str = "git+https://github.com/Uomocosa/lele-rust-linter?tag=v0.0.14";
    const SHA: &str = "8f850b870fbada6ee5a4b346423382a69d13e1d2";
    ["atomic-delegate-macros", "lele-snake-case"]
        .into_iter()
        .map(|name| {
            (
                name.to_string(),
                (String::from("0.0.14"), format!("{GIT}#{SHA}")),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::fleet_lock_sources;

    #[test]
    fn test_usage() {
        let sources = fleet_lock_sources();
        assert_eq!(sources.len(), 2);
        assert_eq!(
            sources
                .get("atomic-delegate-macros")
                .map(|(version, _)| version.as_str()),
            Some("0.0.14")
        );
    }
}
