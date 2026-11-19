use crate::Error;
use crate::fleet::Fleet;

pub fn status(fleet: &Fleet) -> Result<(), Error> {
    println!("lele-release · root {}", fleet.root.display());
    println!();
    fleet.print_repos()?;
    println!();
    fleet.print_pins()?;
    println!();
    fleet.print_filter();
    println!();
    fleet.print_shims();
    println!();
    fleet.print_plan()
}

#[cfg(test)]
mod tests {
    use semver::Version;

    use super::status;
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

        let result = status(&fleet);

        assert!(result.is_ok());
    }
}
