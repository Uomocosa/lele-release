use crate::Error;
use crate::basic;
use crate::fleet::Fleet;
use crate::git::Git;
use crate::repo::Repo;

pub fn status(fleet: &Fleet) -> Result<(), Error> {
    println!("lele-release · root {}", fleet.root.display());
    println!();
    print_repos(fleet)?;
    println!();
    print_pins(fleet)?;
    println!();
    print_filter(fleet);
    println!();
    print_plan(fleet)
}

// needed helper: one row per repo (version, tag, commits since tag, tree state)
fn print_repos(fleet: &Fleet) -> Result<(), Error> {
    let width = fleet
        .repos
        .iter()
        .map(|repo| repo.name.len())
        .max()
        .unwrap_or(0)
        .max(12);
    println!(
        "{:<width$}  {:>7}  {:<8}  {:>5}  tree",
        "repo", "version", "tag", "since"
    );
    for repo in &fleet.repos {
        let tag = repo.latest_tag()?.unwrap_or_else(|| String::from("-"));
        let since = repo
            .commits_since_tag()?
            .map_or_else(|| String::from("-"), |count| count.to_string());
        println!(
            "{:<width$}  {:>7}  {:<8}  {:>5}  {}",
            repo.name,
            repo.version,
            tag,
            since,
            tree_state(repo)
        );
    }
    Ok(())
}

// needed helper: cleanliness and ahead/behind against the remote-tracking branch
fn tree_state(repo: &Repo) -> String {
    let dirty = Git::output(&repo.dir, &["status", "--porcelain"]).unwrap_or_default();
    let counts = Git::output(
        &repo.dir,
        &["rev-list", "--left-right", "--count", "origin/main...HEAD"],
    )
    .unwrap_or_default();
    let mut parts = vec![if dirty.is_empty() {
        String::from("clean")
    } else {
        String::from("DIRTY")
    }];
    let mut numbers = counts.split_whitespace();
    match (numbers.next(), numbers.next()) {
        (Some(behind), Some(ahead)) => {
            if behind == "0" && ahead == "0" {
                parts.push(String::from("in sync"));
            } else {
                parts.push(format!("behind {behind} ahead {ahead}"));
            }
        }
        _ => parts.push(String::from("no upstream")),
    }
    parts.join(" · ")
}

// needed helper: pins grouped by the repo that declares them
fn print_pins(fleet: &Fleet) -> Result<(), Error> {
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
                form_label(&pin.form),
                pin_state(fleet, &pin)
            );
        }
    }
    Ok(())
}

// needed helper: human label for a pin form
fn form_label(form: &basic::enums::PinForm) -> String {
    match form {
        basic::enums::PinForm::Tag(tag) => format!("tag {tag}"),
        basic::enums::PinForm::Rev(rev) => {
            format!("rev {}", rev.chars().take(7).collect::<String>())
        }
        basic::enums::PinForm::Branch(branch) => format!("branch {branch}"),
        basic::enums::PinForm::Path(path) => format!("path {path}"),
        basic::enums::PinForm::Git(url) => format!("git {url}"),
    }
}

// needed helper: ok when the pin resolves to the dependency's latest tag
fn pin_state(fleet: &Fleet, pin: &basic::structs::Pin) -> String {
    if matches!(pin.form, basic::enums::PinForm::Path(_)) {
        return String::from("linked");
    }
    let Some(dependency) = fleet.repos.iter().find(|repo| {
        repo.packages
            .iter()
            .any(|package| package.name == pin.dependency)
    }) else {
        return String::from("unknown");
    };
    let Ok(Some(tag)) = dependency.latest_tag() else {
        return String::from("no tag");
    };
    match &pin.form {
        basic::enums::PinForm::Tag(pinned) if *pinned == tag => String::from("ok"),
        basic::enums::PinForm::Tag(_) => format!("behind {tag}"),
        basic::enums::PinForm::Rev(rev) => {
            let resolved = Git::output(&dependency.dir, &["rev-parse", &tag]).unwrap_or_default();
            if resolved.starts_with(rev.as_str()) {
                String::from("ok")
            } else {
                format!("behind {tag}")
            }
        }
        _ => String::from("?"),
    }
}

// needed helper: filter installation per repo
fn print_filter(fleet: &Fleet) {
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

// needed helper: what a release right now would do, without doing it
fn print_plan(fleet: &Fleet) -> Result<(), Error> {
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
    use super::status;

    #[test]
    fn test_usage() {
        let _ = status;
    }
}
