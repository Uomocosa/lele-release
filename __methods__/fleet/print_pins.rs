use crate::Error;
use crate::fleet::Fleet;

pub fn print_pins(fleet: &Fleet) -> Result<(), Error> {
    println!("pins");
    for repo in &fleet.repos {
        let collected = fleet.pins(repo)?;
        if collected.is_empty() {
            continue;
        }
        println!("  {}", repo.name);
        for pin in collected {
            println!(
                "    {:<28} {:<22} {}",
                pin.dependency,
                Fleet::form_label(&pin.form),
                fleet.pin_state(&pin)
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use semver::Version;

    use super::print_pins;
    use crate::testing;

    #[test]
    fn test_usage() {
        let tmp = tempfile::tempdir().unwrap();
        let dependent_dir = tmp.path().join("dependent");
        let dependent = testing::committed_repo(
            &dependent_dir,
            "dependent",
            Version::new(0, 0, 1),
            Some("v0.0.1"),
        );
        let dependency = testing::committed_repo(
            &tmp.path().join("dep"),
            "dep",
            Version::new(0, 0, 2),
            Some("v0.0.2"),
        );
        std::fs::write(
            dependent_dir.join("Cargo.toml"),
            "[package]\nname = \"dependent\"\nversion = \"0.0.1\"\n\n[dependencies]\ndep = { path = \"../dep\" }\n",
        )
        .unwrap();
        let fleet = testing::fleet(tmp.path(), vec![dependent, dependency], Vec::new());

        let result = print_pins(&fleet);

        assert!(result.is_ok());
    }
}
