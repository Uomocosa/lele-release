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
    use super::print_pins;

    #[test]
    fn test_usage() {
        let _ = print_pins;
    }
}
