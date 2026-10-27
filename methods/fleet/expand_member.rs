use std::path::{Path, PathBuf};

#[must_use]
pub fn expand_member(base: &Path, pattern: &str) -> Vec<PathBuf> {
    if let Some(prefix) = pattern.strip_suffix("/*") {
        let directory = base.join(prefix);
        let mut found = Vec::new();
        let Ok(entries) = std::fs::read_dir(&directory) else {
            return found;
        };
        for entry in entries.flatten() {
            let manifest = entry.path().join("Cargo.toml");
            if manifest.exists() {
                found.push(manifest);
            }
        }
        return found;
    }
    let manifest = base.join(pattern).join("Cargo.toml");
    if manifest.exists() {
        vec![manifest]
    } else {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::expand_member;

    #[test]
    fn test_usage() {
        let _ = expand_member;
    }
}
