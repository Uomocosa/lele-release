use crate::Error;
use crate::basic;
use crate::fleet::Fleet;

pub fn plan(
    fleet: &Fleet,
    level: basic::enums::Bump,
) -> Result<Vec<basic::structs::ReleaseStep>, Error> {
    let changed = fleet.changed_repos()?;
    let affected = fleet.with_transitive_dependents(&changed);
    let ordered = fleet.dependency_first_order(&affected)?;
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
    use super::plan;

    #[test]
    fn test_usage() {
        let _ = plan;
    }
}
