use crate::fleet::Fleet;
use crate::git::Git;

pub fn print_filter(fleet: &Fleet) {
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

#[cfg(test)]
mod tests {
    use super::print_filter;

    #[test]
    fn test_usage() {
        let _ = print_filter;
    }
}
