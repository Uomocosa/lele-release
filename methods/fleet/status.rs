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
    use super::status;

    #[test]
    fn test_usage() {
        let _ = status;
    }
}
