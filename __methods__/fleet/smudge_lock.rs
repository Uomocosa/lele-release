use std::collections::HashMap;
use std::path::Path;

use crate::Error;
use crate::fleet::Fleet;

pub fn smudge_lock(fleet: &Fleet, lock: &Path, input: &str) -> Result<String, Error> {
    let Some(current) = fleet.repos.iter().find(|repo| lock.starts_with(&repo.dir)) else {
        return Ok(input.to_string());
    };
    let mut versions: HashMap<String, String> = HashMap::new();
    for repo in fleet.repos.iter().filter(|repo| repo.name != current.name) {
        for package in &repo.packages {
            versions.insert(package.name.clone(), repo.version.to_string());
        }
    }
    Ok(current.smudge_lock_text(input, &versions))
}

#[cfg(test)]
mod tests {
    use super::smudge_lock;

    #[test]
    fn test_usage() {
        let _ = smudge_lock;
    }
}
