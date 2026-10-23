use semver::Version;

use crate::Error;
use crate::basic;
use crate::fleet::Fleet;

pub fn plan(
    fleet: &Fleet,
    level: basic::enums::Bump,
) -> Result<Vec<basic::structs::ReleaseStep>, Error> {
    let changed = changed_repos(fleet)?;
    let affected = closure(fleet, &changed);
    let ordered = order(fleet, &affected)?;
    let mut steps = Vec::new();
    for name in ordered {
        let Some(repo) = fleet.repos.iter().find(|repo| repo.name == name) else {
            continue;
        };
        let untagged = repo.latest_tag()?.is_none();
        let reason = if untagged {
            String::from("first release")
        } else {
            changed
                .iter()
                .find(|(candidate, _)| *candidate == name)
                .map_or_else(
                    || String::from("pin"),
                    |(_, count)| format!("{count} commit(s) since tag"),
                )
        };
        let next = if untagged {
            repo.version.clone()
        } else {
            bumped(&repo.version, level)
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

// needed helper: repos with their commit count since the latest tag (no tag counts as changed)
fn changed_repos(fleet: &Fleet) -> Result<Vec<(String, u64)>, Error> {
    let mut changed = Vec::new();
    for repo in &fleet.repos {
        match (repo.latest_tag()?, repo.commits_since_tag()?) {
            (None, _) | (Some(_), None) => changed.push((repo.name.clone(), 0)),
            (Some(_), Some(0)) => {}
            (Some(_), Some(count)) => changed.push((repo.name.clone(), count)),
        }
    }
    Ok(changed)
}

// needed helper: changed repos plus every transitive dependent
fn closure(fleet: &Fleet, changed: &[(String, u64)]) -> Vec<String> {
    let mut affected: Vec<String> = changed.iter().map(|(name, _)| name.clone()).collect();
    loop {
        let mut grown = false;
        for edge in &fleet.dependencies {
            if affected.contains(&edge.dependency) && !affected.contains(&edge.dependent) {
                affected.push(edge.dependent.clone());
                grown = true;
            }
        }
        if !grown {
            return affected;
        }
    }
}

// needed helper: dependency-first order (Kahn); a cycle is a refusal, not a guess
fn order(fleet: &Fleet, affected: &[String]) -> Result<Vec<String>, Error> {
    let mut ordered: Vec<String> = Vec::new();
    let mut remaining: Vec<String> = affected.to_vec();
    while !remaining.is_empty() {
        let (ready, rest): (Vec<String>, Vec<String>) = std::mem::take(&mut remaining)
            .into_iter()
            .partition(|name| {
                !fleet.dependencies.iter().any(|edge| {
                    edge.dependent == *name
                        && affected.contains(&edge.dependency)
                        && !ordered.contains(&edge.dependency)
                })
            });
        if ready.is_empty() {
            return Err(Error::Refused(format!(
                "dependency cycle among: {}",
                remaining.join(", ")
            )));
        }
        ordered.extend(ready);
        remaining = rest;
    }
    Ok(ordered)
}

// needed helper: the next version for a bump level
const fn bumped(version: &Version, level: basic::enums::Bump) -> Version {
    match level {
        basic::enums::Bump::Patch => Version::new(
            version.major,
            version.minor,
            version.patch.saturating_add(1),
        ),
        basic::enums::Bump::Minor => {
            Version::new(version.major, version.minor.saturating_add(1), 0)
        }
        basic::enums::Bump::Major => Version::new(version.major.saturating_add(1), 0, 0),
    }
}

#[cfg(test)]
mod tests {
    use semver::Version;

    use super::bumped;
    use crate::basic;

    #[test]
    fn test_usage() {
        let current = Version::parse("0.0.7").unwrap();
        assert_eq!(
            bumped(&current, basic::enums::Bump::Patch),
            Version::parse("0.0.8").unwrap()
        );
        assert_eq!(
            bumped(&current, basic::enums::Bump::Minor),
            Version::parse("0.1.0").unwrap()
        );
        assert_eq!(
            bumped(&current, basic::enums::Bump::Major),
            Version::parse("1.0.0").unwrap()
        );
        let stable = Version::parse("1.4.2").unwrap();
        assert_eq!(
            bumped(&stable, basic::enums::Bump::Patch),
            Version::parse("1.4.3").unwrap()
        );
    }
}
