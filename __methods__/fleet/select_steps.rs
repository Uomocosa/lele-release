use crate::Error;
use crate::basic;
use crate::fleet::Fleet;

pub fn select_steps(
    fleet: &Fleet,
    steps: Vec<basic::structs::ReleaseStep>,
    selection: &basic::enums::Selection,
) -> Result<Vec<basic::structs::ReleaseStep>, Error> {
    match selection {
        basic::enums::Selection::All => Ok(steps),
        basic::enums::Selection::Only(names) => {
            for name in names {
                if !fleet.repos.iter().any(|repo| &repo.name == name) {
                    return Err(Error::Refused(format!("unknown repo `{name}`")));
                }
            }
            Ok(steps
                .into_iter()
                .filter(|step| names.contains(&step.repo))
                .collect())
        }
        basic::enums::Selection::Exclude(names) => Ok(steps
            .into_iter()
            .filter(|step| !names.contains(&step.repo))
            .collect()),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::select_steps;
    use crate::Error;
    use crate::Fleet;
    use crate::Selection;
    use crate::testing;

    #[test]
    fn test_usage() {
        let fleet = Fleet {
            root: PathBuf::new(),
            repos: Vec::new(),
            dependencies: Vec::new(),
        };
        let steps = vec![testing::release_step("a"), testing::release_step("b")];
        let all = select_steps(&fleet, steps.clone(), &Selection::All).unwrap();
        assert_eq!(all.len(), 2);
        let excluded =
            select_steps(&fleet, steps.clone(), &Selection::Exclude(vec!["a".into()])).unwrap();
        assert_eq!(excluded.len(), 1);
        assert_eq!(excluded.first().map(|s| s.repo.as_str()), Some("b"));
        let unknown = select_steps(&fleet, steps, &Selection::Only(vec!["zzz".into()]));
        assert!(matches!(unknown, Err(Error::Refused(_))));
    }
}
