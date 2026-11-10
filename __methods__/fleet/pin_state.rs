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
    use super::pin_state;

    #[test]
    fn test_usage() {
        let _ = pin_state;
    }
}
