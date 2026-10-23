use crate::__basic__;
use crate::Error;
use crate::fleet::Fleet;
use crate::repo::Repo;

pub fn release(
    fleet: &Fleet,
    level: __basic__::enums::Bump,
    selection: &__basic__::enums::Selection,
    dry_run: bool,
) -> Result<(), Error> {
    let steps = select(fleet, fleet.plan(level)?, selection)?;
    if steps.is_empty() {
        println!("nothing to release — every repo matches its tag and every pin is current");
        return Ok(());
    }
    for step in &steps {
        let repo = repo_of(fleet, &step.repo)?;
        repo.ensure_ready()?;
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
        let repo = repo_of(&fresh, &step.repo)?;
        Repo::release(repo, &step.next, &fresh)?;
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

// needed helper: apply --only / --exclude to the planned steps
fn select(
    fleet: &Fleet,
    steps: Vec<__basic__::structs::ReleaseStep>,
    selection: &__basic__::enums::Selection,
) -> Result<Vec<__basic__::structs::ReleaseStep>, Error> {
    match selection {
        __basic__::enums::Selection::All => Ok(steps),
        __basic__::enums::Selection::Only(names) => {
            for name in names {
                if !fleet.repos.iter().any(|repo| &repo.name == name) {
                    return Err(Error::Refused(format!("unknown repo `{name}`")));
                }
            }
            Ok(steps
                .into_iter()
                .filter(|step| names.contains(&step.repo))
                .collect())
        }
        __basic__::enums::Selection::Exclude(names) => Ok(steps
            .into_iter()
            .filter(|step| !names.contains(&step.repo))
            .collect()),
    }
}

// needed helper: look up a repo by name
fn repo_of<'a>(fleet: &'a Fleet, name: &str) -> Result<&'a Repo, Error> {
    fleet
        .repos
        .iter()
        .find(|repo| repo.name == name)
        .ok_or_else(|| Error::Refused(format!("unknown repo `{name}`")))
}

#[cfg(test)]
mod tests {
    use super::release;

    #[test]
    fn test_usage() {
        let _ = release;
    }
}
