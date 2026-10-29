use crate::Error;
use crate::basic;
use crate::fleet::Fleet;

pub fn release(
    fleet: &Fleet,
    level: basic::enums::Bump,
    selection: &basic::enums::Selection,
    dry_run: bool,
    no_install: bool,
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
        if steps.iter().any(|step| step.repo == "lele-release") {
            if no_install {
                println!("install: skipped (--no-install)");
            } else {
                println!("install: lele-release via just install (after all repos)");
            }
        }
        return Ok(());
    }
    println!();
    let (completed, loop_error) = fleet.run_release_loop(&steps);
    let mut pending_error: Option<Error> = None;
    if loop_error.is_none() {
        match Fleet::try_from(fleet.root.as_path()) {
            Ok(fresh) => match fresh.plan(level) {
                Ok(remaining) => {
                    if remaining.is_empty() {
                        println!("done: nothing left behind");
                    } else {
                        let names: Vec<String> =
                            remaining.iter().map(|step| step.repo.clone()).collect();
                        println!(
                            "note: {} repo(s) still pending: {}",
                            remaining.len(),
                            names.join(", ")
                        );
                    }
                }
                Err(error) => pending_error = Some(error),
            },
            Err(error) => pending_error = Some(error),
        }
    }
    let (installed, install_error) = fleet.install_self(&completed, no_install);
    if !installed {
        fleet.warn_if_behind();
    }
    if let Some(error) = loop_error {
        return Err(error);
    }
    if let Some(error) = pending_error {
        return Err(error);
    }
    if let Some(error) = install_error {
        return Err(error);
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
