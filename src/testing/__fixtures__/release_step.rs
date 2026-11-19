use semver::Version;

use crate::ReleaseStep;

pub fn release_step(repo: &str) -> ReleaseStep {
    ReleaseStep {
        repo: repo.to_string(),
        current: Version::new(0, 0, 1),
        next: Version::new(0, 0, 2),
        reason: String::new(),
    }
}

#[cfg(test)]
mod tests {
    use semver::Version;

    use super::release_step;

    #[test]
    fn test_usage() {
        let step = release_step("probe");
        assert_eq!(step.repo, "probe");
        assert_eq!(step.current, Version::new(0, 0, 1));
        assert_eq!(step.next, Version::new(0, 0, 2));
    }
}
