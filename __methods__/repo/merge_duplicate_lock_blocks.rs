use std::fmt::Write as _;

use crate::repo::Repo;

#[must_use]
pub fn merge_duplicate_lock_blocks(_current: &Repo, blocks: Vec<String>) -> Vec<String> {
    let strip_sha = |source: &str| source.split('#').next().unwrap_or(source).to_string();
    let field = |block: &str, key: &str| -> Option<String> {
        block.lines().find_map(|line| {
            line.strip_prefix(key)?
                .strip_suffix('"')
                .map(str::to_string)
        })
    };
    let ids: Vec<Option<(String, String, Option<String>)>> = blocks
        .iter()
        .map(|block| {
            let name = field(block, "name = \"")?;
            Some((
                name,
                field(block, "version = \"")?,
                field(block, "source = \""),
            ))
        })
        .collect();
    let first_of: Vec<Option<usize>> = ids
        .iter()
        .enumerate()
        .map(|(index, id)| {
            let id = id.as_ref()?;
            ids.iter()
                .position(|other| other.as_ref() == Some(id))
                .filter(|&at| at != index)
        })
        .collect();
    if first_of.iter().all(Option::is_none) {
        return blocks;
    }
    let finals: Vec<&(String, String, Option<String>)> = ids
        .iter()
        .zip(&first_of)
        .filter(|(_, dup)| dup.is_none())
        .filter_map(|(id, _)| id.as_ref())
        .collect();
    let canonical = |entry: &str| -> String {
        let mut words = entry.splitn(3, ' ');
        let name = words.next().unwrap_or_default();
        let version = words.next();
        let source = words.next().map(|s| strip_sha(s.trim_matches(['(', ')'])));
        let named = finals.iter().copied().filter(|(n, _, _)| n == name);
        let by_name: Vec<&(String, String, Option<String>)> = named.collect();
        let narrowed = by_name.iter().copied().filter(|(_, v, src)| {
            version.is_none_or(|wanted| v == wanted)
                && (source.is_none() || src.as_deref().map(strip_sha) == source)
        });
        let narrowed: Vec<&(String, String, Option<String>)> = narrowed.collect();
        let (([only], _) | (_, [only])) = (by_name.as_slice(), narrowed.as_slice()) else {
            return entry.to_string();
        };
        let (n, v, src) = only;
        let same_version = by_name.iter().filter(|(_, other, _)| other == v).count();
        match (by_name.len(), same_version, src) {
            (1, _, _) => n.clone(),
            (_, 1, _) | (_, _, None) => format!("{n} {v}"),
            (_, _, Some(src)) => format!("{n} {v} ({})", strip_sha(src)),
        }
    };
    let rewrite = |block: &str| -> String {
        let mut out = String::new();
        for line in block.lines() {
            let entry = line
                .strip_prefix(" \"")
                .and_then(|rest| rest.strip_suffix("\","));
            let _ = match entry {
                Some(entry) => writeln!(out, " \"{}\",", canonical(entry)),
                None => writeln!(out, "{line}"),
            };
        }
        out
    };
    let rewritten: Vec<String> = blocks.iter().map(|block| rewrite(block)).collect();
    let conflict = first_of
        .iter()
        .enumerate()
        .any(|(index, dup)| dup.is_some_and(|first| rewritten.get(index) != rewritten.get(first)));
    if conflict {
        return blocks;
    }
    let kept = rewritten
        .into_iter()
        .zip(&first_of)
        .filter(|(_, dup)| dup.is_none());
    let mut kept: Vec<String> = kept.map(|(block, _)| block).collect();
    if let Some(last) = kept.last_mut() {
        *last = format!("{}\n", last.trim_end());
    }
    kept
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use semver::Version;

    use super::merge_duplicate_lock_blocks;
    use crate::testing;

    const FIXTURE: &str = include_str!("../../fixtures/plugin_406155c_duplicate_blocks.lock");

    #[test]
    fn test_usage() {
        let repo = testing::repo(
            "probe",
            Path::new("/nonexistent/probe"),
            "",
            "common_types",
            Version::new(0, 0, 11),
        );
        let block = String::from("[[package]]\nname = \"twin\"\nversion = \"1.0.0\"\n\n");

        let merged = merge_duplicate_lock_blocks(&repo, vec![block.clone(), block]);

        assert_eq!(merged.len(), 1);
    }

    #[test]
    fn test_clean_merges_path_and_git_blocks() {
        let repo = testing::repo(
            "probe",
            Path::new("/nonexistent/probe"),
            "",
            "common_types",
            Version::new(0, 0, 11),
        );
        let cleaned = repo.clean_lock_text(FIXTURE, &testing::fleet_lock_sources());
        assert_eq!(
            testing::lock_package_block_count(&cleaned, "atomic-delegate-macros"),
            1
        );
        assert_eq!(
            testing::lock_package_block_count(&cleaned, "lele-snake-case"),
            1
        );
        assert!(!cleaned.contains("lele-snake-case 0.0.14"));
        assert!(!cleaned.contains("atomic-delegate-macros 0.0.14"));
        assert!(cleaned.contains(" \"atomic-delegate-macros\",\n"));
        assert!(cleaned.contains(" \"lele-snake-case\",\n"));
        assert!(cleaned.ends_with("]\n"));
    }

    #[test]
    fn test_clean_is_idempotent() {
        let repo = testing::repo(
            "probe",
            Path::new("/nonexistent/probe"),
            "",
            "common_types",
            Version::new(0, 0, 11),
        );
        let once = repo.clean_lock_text(FIXTURE, &testing::fleet_lock_sources());
        assert_eq!(
            repo.clean_lock_text(&once, &testing::fleet_lock_sources()),
            once
        );
    }

    #[test]
    fn test_smudge_of_clean_has_no_duplicate_identity() {
        let repo = testing::repo(
            "probe",
            Path::new("/nonexistent/probe"),
            "",
            "common_types",
            Version::new(0, 0, 11),
        );
        let cleaned = repo.clean_lock_text(FIXTURE, &testing::fleet_lock_sources());
        let smudged = repo.smudge_lock_text(&cleaned, &testing::fleet_lock_versions());
        assert_eq!(
            testing::lock_package_block_count(&smudged, "atomic-delegate-macros"),
            1
        );
        assert_eq!(
            testing::lock_package_block_count(&smudged, "lele-snake-case"),
            1
        );
        assert!(!smudged.contains("source = \"git+https://github.com/Uomocosa/lele-rust-linter"));
        assert_eq!(
            repo.smudge_lock_text(&smudged, &testing::fleet_lock_versions()),
            smudged
        );
    }

    #[test]
    fn test_smudge_merges_blocks_that_become_identical() {
        let repo = testing::repo(
            "probe",
            Path::new("/nonexistent/probe"),
            "",
            "common_types",
            Version::new(0, 0, 11),
        );
        let smudged = repo.smudge_lock_text(FIXTURE, &testing::fleet_lock_versions());
        assert_eq!(
            testing::lock_package_block_count(&smudged, "atomic-delegate-macros"),
            1
        );
        assert_eq!(
            testing::lock_package_block_count(&smudged, "lele-snake-case"),
            1
        );
    }

    #[test]
    fn test_two_versions_of_non_fleet_package_are_untouched() {
        let repo = testing::repo(
            "probe",
            Path::new("/nonexistent/probe"),
            "",
            "common_types",
            Version::new(0, 0, 11),
        );
        let lock = "version = 4\n\n[[package]]\nname = \"syn\"\nversion = \"1.0.109\"\n\n[[package]]\nname = \"syn\"\nversion = \"2.0.119\"\n\n[[package]]\nname = \"user\"\nversion = \"0.1.0\"\ndependencies = [\n \"syn 1.0.109\",\n \"syn 2.0.119\",\n]\n";
        assert_eq!(
            repo.clean_lock_text(lock, &testing::fleet_lock_sources()),
            lock
        );
        assert_eq!(
            repo.smudge_lock_text(lock, &testing::fleet_lock_versions()),
            lock
        );
    }

    #[test]
    fn test_blocks_with_different_dependencies_are_left_alone() {
        let repo = testing::repo(
            "probe",
            Path::new("/nonexistent/probe"),
            "",
            "common_types",
            Version::new(0, 0, 11),
        );
        let block = |dep: &str| {
            format!(
                "[[package]]\nname = \"twin\"\nversion = \"1.0.0\"\ndependencies = [\n \"{dep}\",\n]\n\n"
            )
        };
        let blocks = vec![block("quote"), block("syn")];
        assert_eq!(merge_duplicate_lock_blocks(&repo, blocks.clone()), blocks);
    }

    #[test]
    fn test_identical_blocks_with_same_dependencies_merge() {
        let repo = testing::repo(
            "probe",
            Path::new("/nonexistent/probe"),
            "",
            "common_types",
            Version::new(0, 0, 11),
        );
        let block = "[[package]]\nname = \"twin\"\nversion = \"1.0.0\"\n\n".to_string();
        let merged = merge_duplicate_lock_blocks(&repo, vec![block.clone(), block.clone()]);
        assert_eq!(merged, vec![format!("{}\n", block.trim_end())]);
    }
}
