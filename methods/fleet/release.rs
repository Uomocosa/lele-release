use crate::Error;
use crate::basic;
use crate::fleet::Fleet;
use crate::repo::Repo;

pub fn release(
    fleet: &Fleet,
    level: basic::enums::Bump,
    selection: &basic::enums::Selection,
    dry_run: bool,
) -> Result<(), Error> {
    let steps = fleet.select_steps(fleet.plan(level)?, selection)?;
    if steps.is_empty() {
        println!("nothing to release — every repo matches its tag and every pin is current");
        return Ok(());
    }
    for step in &steps {
        let repo = fleet.repo_named(&step.repo)?;
        repo.ensure_ready(fleet)?;
        println!("preflight ✓ {:<28} clean · main · in sync", repo.name);
    }
    println!();
    println!("AFFECTED ({})", steps.len());
    for step in &steps {
        println!(
            "  {:<28} {} -> {}   ({})",
            step.repo, step.current, step.next, step.reason
        );
    }
    if dry_run {
        println!();
        println!("dry run: nothing written");
        return Ok(());
    }
    println!();
    for step in &steps {
        let fresh = Fleet::try_from(fleet.root.as_path())?;
        let repo = fresh.repo_named(&step.repo)?;
        Repo::release(repo, &step.next)?;
        println!(
            "[{}] {:<28} {} · tag v{} · pushed",
            step.repo, step.current, step.next, step.next
        );
    }
    let remaining = Fleet::try_from(fleet.root.as_path())?.plan(level)?;
    if remaining.is_empty() {
        println!("done: nothing left behind");
    } else {
        let names: Vec<String> = remaining.iter().map(|step| step.repo.clone()).collect();
        println!(
            "note: {} repo(s) still pending: {}",
            remaining.len(),
            names.join(", ")
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::release;

    #[test]
    fn test_usage() {
        let _ = release;
    }
}
