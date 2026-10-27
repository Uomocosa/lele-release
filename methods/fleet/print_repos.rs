use crate::Error;
use crate::fleet::Fleet;

pub fn print_repos(fleet: &Fleet) -> Result<(), Error> {
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
            repo.tree_state()
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::print_repos;

    #[test]
    fn test_usage() {
        let _ = print_repos;
    }
}
