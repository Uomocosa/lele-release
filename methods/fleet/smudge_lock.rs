use std::collections::HashMap;
use std::fmt::Write as _;
use std::path::Path;

use crate::Error;
use crate::fleet::Fleet;
use crate::repo::Repo;

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
    Ok(rewrite(input, current, &versions))
}

// needed helper: put every fleet entry in the working-tree (path) form, key order preserved
fn rewrite(input: &str, current: &Repo, versions: &HashMap<String, String>) -> String {
    let mut out = String::new();
    let mut block = String::new();
    for line in input.lines() {
        if line == "[[package]]" && !block.is_empty() {
            out.push_str(&rewrite_block(&block, current, versions));
            block.clear();
        }
        block.push_str(line);
        block.push('\n');
    }
    if !block.is_empty() {
        out.push_str(&rewrite_block(&block, current, versions));
    }
    out
}

// needed helper: one `[[package]]` block
fn rewrite_block(block: &str, current: &Repo, versions: &HashMap<String, String>) -> String {
    let Some(name) = block
        .lines()
        .find_map(|line| line.strip_prefix("name = \"")?.strip_suffix('"'))
    else {
        return block.to_string();
    };
    let desired = if current.packages.iter().any(|package| package.name == name) {
        Some(current.version.to_string())
    } else {
        versions.get(name).cloned()
    };
    let Some(version) = desired else {
        return block.to_string();
    };
    let mut out = String::new();
    for line in block.lines() {
        if line.starts_with("source = ") {
            continue;
        }
        if line.starts_with("version = ") {
            let _ = writeln!(out, "version = \"{version}\"");
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::smudge_lock;

    #[test]
    fn test_usage() {
        let _ = smudge_lock;
    }
}
