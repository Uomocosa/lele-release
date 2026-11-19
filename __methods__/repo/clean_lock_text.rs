use std::collections::HashMap;
use std::hash::BuildHasher;

use crate::repo::Repo;

#[must_use]
pub fn clean_lock_text<S: BuildHasher>(
    current: &Repo,
    input: &str,
    sources: &HashMap<String, (String, String), S>,
) -> String {
    let mut out: Vec<String> = Vec::new();
    let mut block = String::new();
    for line in input.lines() {
        if line == "[[package]]" && !block.is_empty() {
            out.push(current.clean_lock_block(&block, sources));
            block.clear();
        }
        block.push_str(line);
        block.push('\n');
    }
    if !block.is_empty() {
        out.push(current.clean_lock_block(&block, sources));
    }
    current.merge_duplicate_lock_blocks(out).concat()
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use semver::Version;

    use super::clean_lock_text;
    use crate::testing;

    const TEXT: &str = "[[package]]\nname = \"probe\"\nversion = \"0.0.1\"\n\n[[package]]\nname = \"other\"\nversion = \"9.9.9\"\n";

    #[test]
    fn test_usage() {
        let repo = testing::repo(
            "probe",
            Path::new("/nonexistent/probe"),
            "",
            "probe",
            Version::new(0, 0, 2),
        );

        let cleaned = clean_lock_text(&repo, TEXT, &testing::fleet_lock_sources());

        assert!(
            cleaned.contains("name = \"probe\"\nversion = \"0.0.2\""),
            "{cleaned}"
        );
        assert!(
            cleaned.contains("name = \"other\"\nversion = \"9.9.9\""),
            "{cleaned}"
        );
    }
}
