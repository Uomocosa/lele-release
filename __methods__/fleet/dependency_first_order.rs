use crate::Error;
use crate::fleet::Fleet;

pub fn dependency_first_order(fleet: &Fleet, affected: &[String]) -> Result<Vec<String>, Error> {
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

#[cfg(test)]
mod tests {
    use super::dependency_first_order;
    use crate::basic;
    use crate::fleet::Fleet;

    #[test]
    fn test_usage() {
        let edge = |dependent: &str, dependency: &str| basic::structs::Dependency {
            dependent: String::from(dependent),
            dependency: String::from(dependency),
        };
        let mut fleet = Fleet {
            root: std::env::temp_dir(),
            repos: Vec::new(),
            dependencies: vec![edge("app", "lib")],
        };
        let affected = vec![String::from("app"), String::from("lib")];
        assert_eq!(
            dependency_first_order(&fleet, &affected).unwrap(),
            vec![String::from("lib"), String::from("app")]
        );
        fleet.dependencies.push(edge("lib", "app"));
        assert!(dependency_first_order(&fleet, &affected).is_err());
    }
}
