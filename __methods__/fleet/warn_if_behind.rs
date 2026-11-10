use crate::fleet::Fleet;

pub fn warn_if_behind(fleet: &Fleet) {
    let Some(repo) = fleet.repos.iter().find(|repo| repo.name == "lele-release") else {
        return;
    };
    let Ok(Some(tag)) = repo.latest_tag() else {
        return;
    };
    let Ok(tagged) = tag.trim_start_matches('v').parse::<semver::Version>() else {
        return;
    };
    let Ok(running) = env!("CARGO_PKG_VERSION").parse::<semver::Version>() else {
        return;
    };
    if running < tagged {
        eprintln!(
            "note: running lele-release {running} but v{tagged} is tagged — run `just install` in lele-release"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::warn_if_behind;

    #[test]
    fn test_usage() {
        let _ = warn_if_behind;
    }
}
