use crate::basic;

#[must_use]
pub fn self_install_due(steps: &[basic::structs::ReleaseStep], no_install: bool) -> bool {
    let mentions_lele = steps.iter().any(|step| step.repo == "lele-release");
    !no_install && mentions_lele
}

#[cfg(test)]
mod tests {
    use semver::Version;

    use super::self_install_due;
    use crate::basic;

    fn step(repo: &str) -> basic::structs::ReleaseStep {
        basic::structs::ReleaseStep {
            repo: String::from(repo),
            current: Version::new(0, 0, 8),
            next: Version::new(0, 0, 9),
            reason: String::from("pin"),
        }
    }

    #[test]
    fn test_usage() {
        assert!(self_install_due(&[step("lele-release")], false));
        assert!(!self_install_due(&[step("other")], false));
        assert!(!self_install_due(&[step("lele-release")], true));
        assert!(!self_install_due(&[], false));
    }
}
