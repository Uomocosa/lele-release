use std::path::{Path, PathBuf};

#[must_use]
pub fn normalise_lexically(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                out.pop();
            }
            std::path::Component::CurDir => {}
            other => out.push(other),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::normalise_lexically;

    #[test]
    fn test_usage() {
        let folded = normalise_lexically(Path::new("/no/such/dir/../sibling/./crate"));
        assert_eq!(folded, PathBuf::from("/no/such/sibling/crate"));
    }
}
