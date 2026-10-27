use std::collections::HashMap;
use std::hash::BuildHasher;

use crate::repo::Repo;

#[must_use]
pub fn clean_lock_text<S: BuildHasher>(
    current: &Repo,
    input: &str,
    sources: &HashMap<String, (String, String), S>,
) -> String {
    let mut out = String::new();
    let mut block = String::new();
    for line in input.lines() {
        if line == "[[package]]" && !block.is_empty() {
            out.push_str(&current.clean_lock_block(&block, sources));
            block.clear();
        }
        block.push_str(line);
        block.push('\n');
    }
    if !block.is_empty() {
        out.push_str(&current.clean_lock_block(&block, sources));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::clean_lock_text;

    #[test]
    fn test_usage() {
        let _ = clean_lock_text::<std::hash::RandomState>;
    }
}
