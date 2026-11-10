use std::collections::HashMap;
use std::fmt::Write as _;
use std::hash::BuildHasher;

use crate::repo::Repo;

#[must_use]
pub fn clean_lock_block<S: BuildHasher>(
    current: &Repo,
    block: &str,
    sources: &HashMap<String, (String, String), S>,
) -> String {
    let Some(name) = block
        .lines()
        .find_map(|line| line.strip_prefix("name = \"")?.strip_suffix('"'))
    else {
        return block.to_string();
    };
    let desired = if current.packages.iter().any(|package| package.name == name) {
        Some((current.version.to_string(), None))
    } else {
        sources
            .get(name)
            .map(|(version, source)| (version.clone(), Some(source.clone())))
    };
    let Some((version, source)) = desired else {
        return block.to_string();
    };
    let mut out = String::new();
    for line in block.lines() {
        if line.starts_with("source = ") {
            continue;
        }
        if line.starts_with("version = ") {
            let _ = writeln!(out, "version = \"{version}\"");
            if let Some(source) = &source {
                let _ = writeln!(out, "source = \"{source}\"");
            }
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::clean_lock_block;

    #[test]
    fn test_usage() {
        let _ = clean_lock_block::<std::hash::RandomState>;
    }
}
