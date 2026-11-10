use std::collections::HashMap;
use std::path::Path;

use crate::Error;
use crate::fleet::Fleet;
use crate::git::Git;

pub fn clean_lock(fleet: &Fleet, lock: &Path, input: &str) -> Result<String, Error> {
    let Some(current) = fleet.repos.iter().find(|repo| lock.starts_with(&repo.dir)) else {
        return Ok(input.to_string());
    };
    let mut sources: HashMap<String, (String, String)> = HashMap::new();
    for repo in fleet.repos.iter().filter(|repo| repo.name != current.name) {
        let Some(tag) = repo.latest_tag()? else {
            continue;
        };
        let sha = Git::output(&repo.dir, &["rev-parse", &format!("{tag}^{{commit}}")])?;
        let version = tag.trim_start_matches('v').to_string();
        let source = format!("git+{}?tag={tag}#{sha}", repo.url.trim_end_matches(".git"));
        for package in &repo.packages {
            sources.insert(package.name.clone(), (version.clone(), source.clone()));
        }
    }
    Ok(current.clean_lock_text(input, &sources))
}

#[cfg(test)]
mod tests {
    use super::clean_lock;

    #[test]
    fn test_usage() {
        let _ = clean_lock;
    }
}
