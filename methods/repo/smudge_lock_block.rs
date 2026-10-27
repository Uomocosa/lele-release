use std::collections::HashMap;
use std::fmt::Write as _;
use std::hash::BuildHasher;

use crate::repo::Repo;

#[must_use]
pub fn smudge_lock_block<S: BuildHasher>(
    current: &Repo,
    block: &str,
    versions: &HashMap<String, String, S>,
) -> String {
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
    use super::smudge_lock_block;

    #[test]
    fn test_usage() {
        let _ = smudge_lock_block::<std::hash::RandomState>;
    }
}
