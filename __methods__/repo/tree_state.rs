use crate::git::Git;
use crate::repo::Repo;

#[must_use]
pub fn tree_state(repo: &Repo) -> String {
    let dirty = Git::output(&repo.dir, &["status", "--porcelain"]).unwrap_or_default();
    let counts = Git::output(
        &repo.dir,
        &["rev-list", "--left-right", "--count", "origin/main...HEAD"],
    )
    .unwrap_or_default();
    let mut parts = vec![if dirty.is_empty() {
        String::from("clean")
    } else {
        String::from("DIRTY")
    }];
    let mut numbers = counts.split_whitespace();
    match (numbers.next(), numbers.next()) {
        (Some(behind), Some(ahead)) => {
            if behind == "0" && ahead == "0" {
                parts.push(String::from("in sync"));
            } else {
                parts.push(format!("behind {behind} ahead {ahead}"));
            }
        }
        _ => parts.push(String::from("no upstream")),
    }
    parts.join(" · ")
}

#[cfg(test)]
mod tests {
    use semver::Version;

    use super::tree_state;
    use crate::testing;

    #[test]
    fn test_usage() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = testing::committed_repo(
            &tmp.path().join("probe"),
            "probe",
            Version::new(0, 0, 1),
            None,
        );

        let state = tree_state(&repo);

        assert_eq!(state, "clean · no upstream");
    }
}
