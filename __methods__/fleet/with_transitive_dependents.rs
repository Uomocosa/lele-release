use crate::fleet::Fleet;

#[must_use]
pub fn with_transitive_dependents(fleet: &Fleet, changed: &[(String, u64)]) -> Vec<String> {
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

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::with_transitive_dependents;
    use crate::basic;
    use crate::testing;

    #[test]
    fn test_usage() {
        let edge = |dependent: &str, dependency: &str| basic::structs::Dependency {
            dependent: String::from(dependent),
            dependency: String::from(dependency),
        };
        let fleet = testing::fleet(
            Path::new("/fleet"),
            Vec::new(),
            vec![edge("middle", "app"), edge("top", "middle")],
        );
        let changed = vec![(String::from("app"), 1_u64)];

        let affected = with_transitive_dependents(&fleet, &changed);

        assert_eq!(
            affected,
            vec![
                String::from("app"),
                String::from("middle"),
                String::from("top"),
            ]
        );
    }
}
