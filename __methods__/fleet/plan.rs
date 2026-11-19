use crate::Error;
use crate::basic;
use crate::fleet::Fleet;

pub fn plan(
    fleet: &Fleet,
    level: basic::enums::Bump,
) -> Result<Vec<basic::structs::ReleaseStep>, Error> {
    let changed = fleet.changed_repos()?;
    let mut behind_reasons: Vec<(String, String)> = Vec::new();
    for repo in &fleet.repos {
        if changed.iter().any(|(name, _)| *name == repo.name) {
            continue;
        }
        let pinned = fleet.pins(repo)?;
        let behind: Vec<(&basic::structs::Pin, String)> = pinned
            .iter()
            .filter(|pin| {
                matches!(
                    pin.form,
                    basic::enums::PinForm::Tag(_) | basic::enums::PinForm::Rev(_)
                )
            })
            .filter_map(|pin| {
                let state = fleet.pin_state(pin);
                state
                    .strip_prefix("behind ")
                    .map(|latest| (pin, latest.to_string()))
            })
            .collect();
        let Some((pin, latest)) = behind.first() else {
            continue;
        };
        let pinned_label = match &pin.form {
            basic::enums::PinForm::Tag(tag) => tag.clone(),
            basic::enums::PinForm::Rev(rev) => rev.chars().take(7).collect::<String>(),
            _ => continue,
        };
        let extra = if behind.len() > 1 {
            format!(" +{}", behind.len().saturating_sub(1))
        } else {
            String::new()
        };
        behind_reasons.push((
            repo.name.clone(),
            format!(
                "pin behind: {} {pinned_label} -> {latest}{extra}",
                pin.dependency
            ),
        ));
    }
    let mut seeds = changed.clone();
    seeds.extend(behind_reasons.iter().map(|(name, _)| (name.clone(), 0_u64)));
    let affected = fleet.with_transitive_dependents(&seeds);
    let ordered = fleet.dependency_first_order(&affected)?;
    let mut steps = Vec::new();
    for name in ordered {
        let Some(repo) = fleet.repos.iter().find(|repo| repo.name == name) else {
            continue;
        };
        let untagged = repo.latest_tag()?.is_none();
        let reason = if untagged {
            String::from("first release")
        } else if let Some((_, count)) = changed.iter().find(|(candidate, _)| *candidate == name) {
            format!("{count} commit(s) since tag")
        } else {
            behind_reasons
                .iter()
                .find(|(candidate, _)| *candidate == name)
                .map_or_else(|| String::from("pin"), |(_, reason)| reason.clone())
        };
        let next = if untagged {
            repo.version.clone()
        } else {
            Fleet::bumped(&repo.version, level)
        };
        steps.push(basic::structs::ReleaseStep {
            repo: name,
            current: repo.version.clone(),
            next,
            reason,
        });
    }
    Ok(steps)
}

#[cfg(test)]
mod tests {
    use semver::Version;

    use super::plan;
    use crate::basic;
    use crate::fleet::Fleet;
    use crate::git::Git;
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
        let fleet = testing::fleet(tmp.path(), vec![repo], Vec::new());

        let steps = plan(&fleet, basic::enums::Bump::Patch).unwrap();

        assert_eq!(steps.len(), 1);
        assert_eq!(
            steps.first().map(|step| step.reason.as_str()),
            Some("first release")
        );
        assert_eq!(
            steps.first().map(|step| step.next.clone()),
            Some(Version::new(0, 0, 1))
        );
    }

    #[test]
    fn plans_a_tagged_repo_whose_fleet_tag_pin_is_behind() {
        let tmp = tempfile::tempdir().unwrap();
        let dependent_dir = tmp.path().join("dependent");
        let sibling_dir = tmp.path().join("sibling");
        std::fs::create_dir_all(&dependent_dir).unwrap();
        std::fs::create_dir_all(&sibling_dir).unwrap();

        std::fs::write(
            sibling_dir.join("Cargo.toml"),
            "[package]\nname = \"dep\"\nversion = \"0.0.1\"\n",
        )
        .unwrap();
        Git::run(&sibling_dir, &["init", "-q", "-b", "main"]).unwrap();
        Git::run(&sibling_dir, &["add", "--", "Cargo.toml"]).unwrap();
        testing::commit_tag(&sibling_dir, "v0.0.1", "v0.0.1");
        std::fs::write(
            sibling_dir.join("Cargo.toml"),
            "[package]\nname = \"dep\"\nversion = \"0.0.2\"\n",
        )
        .unwrap();
        Git::run(&sibling_dir, &["add", "--", "Cargo.toml"]).unwrap();
        testing::commit_tag(&sibling_dir, "v0.0.2", "v0.0.2");

        std::fs::write(
            dependent_dir.join("Cargo.toml"),
            "[package]\nname = \"dependent\"\nversion = \"0.0.1\"\n\n[dependencies]\ndep = { git = \"https://example.com/dep.git\", tag = \"v0.0.1\" }\n\n[dev-dependencies]\ndep = { git = \"https://example.com/dep.git\", tag = \"v0.0.1\" }\n",
        )
        .unwrap();
        Git::run(&dependent_dir, &["init", "-q", "-b", "main"]).unwrap();
        Git::run(&dependent_dir, &["add", "--", "Cargo.toml"]).unwrap();
        testing::commit_tag(&dependent_dir, "v0.0.1", "v0.0.1");

        let fleet = Fleet {
            root: tmp.path().to_path_buf(),
            repos: vec![
                testing::repo(
                    "dependent",
                    &dependent_dir,
                    "https://example.com/dependent.git",
                    "dependent",
                    Version::new(0, 0, 1),
                ),
                testing::repo(
                    "dep",
                    &sibling_dir,
                    "https://example.com/dep.git",
                    "dep",
                    Version::new(0, 0, 2),
                ),
            ],
            dependencies: vec![basic::structs::Dependency {
                dependent: String::from("dependent"),
                dependency: String::from("dep"),
            }],
        };

        let steps = plan(&fleet, basic::enums::Bump::Patch).unwrap();
        assert_eq!(steps.len(), 1);
        let step = steps.first().unwrap();
        assert_eq!(step.repo, "dependent");
        assert_eq!(step.current, Version::new(0, 0, 1));
        assert_eq!(step.next, Version::new(0, 0, 2));
        assert_eq!(step.reason, "pin behind: dep v0.0.1 -> v0.0.2 +1");
    }
}
