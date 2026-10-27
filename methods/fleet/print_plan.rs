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
    use super::print_plan;

    #[test]
    fn test_usage() {
        let _ = print_plan;
    }
}
