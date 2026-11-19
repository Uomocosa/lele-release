use crate::repo::Repo;

#[must_use]
pub fn matches_url(repo: &Repo, url: &str) -> bool {
    let left = Repo::slug(&repo.url);
    let right = Repo::slug(url);
    !left.is_empty() && left == right
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use semver::Version;

    use super::matches_url;
    use crate::testing;

    #[test]
    fn test_usage() {
        let repo = testing::repo(
            "probe",
            Path::new("/nonexistent/probe"),
            "https://github.com/Uomocosa/lele-release.git",
            "probe",
            Version::new(0, 0, 1),
        );

        let same = matches_url(&repo, "git@github.com:uomocosa/lele-release");
        let other = matches_url(&repo, "https://github.com/Uomocosa/other.git");

        assert!(same);
        assert!(!other);
    }
}
