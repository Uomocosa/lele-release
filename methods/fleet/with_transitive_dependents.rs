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
    use super::with_transitive_dependents;

    #[test]
    fn test_usage() {
        let _ = with_transitive_dependents;
    }
}
