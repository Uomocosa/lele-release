use crate::basic;

#[must_use]
pub fn self_install_due(steps: &[basic::structs::ReleaseStep], no_install: bool) -> bool {
    let mentions_lele = steps.iter().any(|step| step.repo == "lele-release");
    !no_install && mentions_lele
}

#[cfg(test)]
mod tests {
    use super::self_install_due;
    use crate::testing;

    #[test]
    fn test_usage() {
        assert!(self_install_due(
            &[testing::release_step("lele-release")],
            false
        ));
        assert!(!self_install_due(&[testing::release_step("other")], false));
        assert!(!self_install_due(
            &[testing::release_step("lele-release")],
            true
        ));
        assert!(!self_install_due(&[], false));
    }
}
