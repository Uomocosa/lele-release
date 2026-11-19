use crate::fleet::Fleet;
use crate::git::Git;

pub fn print_filter(fleet: &Fleet) {
    let mut missing: Vec<&str> = Vec::new();
    for repo in &fleet.repos {
        let installed =
            Git::output(&repo.dir, &["config", "--get", "filter.lele-pins.clean"]).is_ok();
        let attribute =
            Git::output(&repo.dir, &["check-attr", "filter", "Cargo.toml"]).unwrap_or_default();
        if !installed || !attribute.contains("lele-pins") {
            missing.push(&repo.name);
        }
        println!(
            "filter  {:<28} driver: {:<9} attribute: {}",
            repo.name,
            if installed { "installed" } else { "missing" },
            if attribute.contains("lele-pins") {
                "set"
            } else {
                "absent"
            }
        );
    }
    if missing.is_empty() {
        println!("working tree: path form (lele-pins active)");
    } else {
        println!(
            "working tree: pinned form (filter missing: {})",
            missing.join(", ")
        );
    }
}

#[cfg(test)]
mod tests {
    use semver::Version;

    use super::print_filter;
    use crate::Git;
    use crate::testing;

    #[test]
    fn test_usage() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = testing::committed_repo(
            &tmp.path().join("probe"),
            "probe",
            Version::new(0, 0, 1),
            None,
        );
        let fleet = testing::fleet(tmp.path(), vec![repo], Vec::new());

        print_filter(&fleet);

        assert_eq!(fleet.repos.len(), 1);
        let dir = fleet.repos.first().unwrap().dir.clone();
        assert!(
            Git::output(&dir, &["config", "--get", "filter.lele-pins.clean"]).is_err(),
            "the report must not install the filter"
        );
    }
}
