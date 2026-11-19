use crate::Error;
use crate::basic;
use crate::fleet::Fleet;

pub fn print_plan(fleet: &Fleet) -> Result<(), Error> {
    let steps = fleet.plan(basic::enums::Bump::Patch)?;
    if steps.is_empty() {
        println!("would release (patch): nothing");
        return Ok(());
    }
    println!("would release (patch):");
    for step in steps {
        println!(
            "  {:<28} {} -> {}  ({})",
            step.repo, step.current, step.next, step.reason
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use semver::Version;

    use super::print_plan;
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

        let result = print_plan(&fleet);

        assert!(result.is_ok());
    }
}
