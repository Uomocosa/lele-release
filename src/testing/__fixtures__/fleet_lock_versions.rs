use std::collections::HashMap;

use crate::testing;

pub fn fleet_lock_versions() -> HashMap<String, String> {
    testing::fleet_lock_sources()
        .into_iter()
        .map(|(name, (version, _source))| (name, version))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::fleet_lock_versions;

    #[test]
    fn test_usage() {
        let versions = fleet_lock_versions();
        assert_eq!(versions.len(), 2);
        assert_eq!(
            versions.get("lele-snake-case").map(String::as_str),
            Some("0.0.14")
        );
    }
}
