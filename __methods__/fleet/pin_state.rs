use crate::basic;
use crate::fleet::Fleet;
use crate::git::Git;

#[must_use]
pub fn pin_state(fleet: &Fleet, pin: &basic::structs::Pin) -> String {
    if matches!(pin.form, basic::enums::PinForm::Path(_)) {
        return String::from("linked");
    }
    let Some(dependency) = fleet.repos.iter().find(|repo| {
        repo.packages
            .iter()
            .any(|package| package.name == pin.dependency)
    }) else {
        return String::from("unknown");
    };
    let Ok(Some(tag)) = dependency.latest_tag() else {
        return String::from("no tag");
    };
    match &pin.form {
        basic::enums::PinForm::Tag(pinned) if *pinned == tag => String::from("ok"),
        basic::enums::PinForm::Tag(_) => format!("behind {tag}"),
        basic::enums::PinForm::Rev(rev) => {
            let resolved = Git::output(&dependency.dir, &["rev-parse", &tag]).unwrap_or_default();
            if resolved.starts_with(rev.as_str()) {
                String::from("ok")
            } else {
                format!("behind {tag}")
            }
        }
        _ => String::from("?"),
    }
}

#[cfg(test)]
mod tests {
    use semver::Version;

    use super::pin_state;
    use crate::basic;
    use crate::testing;

    #[test]
    fn test_usage() {
        let tmp = tempfile::tempdir().unwrap();
        let dependency = testing::committed_repo(
            &tmp.path().join("dep"),
            "dep",
            Version::new(0, 0, 2),
            Some("v0.0.2"),
        );
        let manifest = dependency.manifests.first().unwrap().clone();
        let fleet = testing::fleet(tmp.path(), vec![dependency], Vec::new());
        let linked = basic::structs::Pin {
            manifest,
            dependent: String::from("app"),
            dependency: String::from("dep"),
            form: basic::enums::PinForm::Path(String::from("../dep")),
        };
        let tagged = basic::structs::Pin {
            form: basic::enums::PinForm::Tag(String::from("v0.0.2")),
            ..linked.clone()
        };

        let linked_state = pin_state(&fleet, &linked);
        let tagged_state = pin_state(&fleet, &tagged);

        assert_eq!(linked_state, "linked");
        assert_eq!(tagged_state, "ok");
    }
}
