use std::path::Path;

use crate::Dependency;
use crate::Fleet;
use crate::Repo;

pub fn fleet(root: &Path, repos: Vec<Repo>, dependencies: Vec<Dependency>) -> Fleet {
    Fleet {
        root: root.to_path_buf(),
        repos,
        dependencies,
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use semver::Version;

    use super::fleet;
    use crate::Dependency;
    use crate::testing;

    #[test]
    fn test_usage() {
        let repo = testing::repo(
            "probe",
            Path::new("/fleet/probe"),
            "https://example.com/probe.git",
            "probe",
            Version::new(0, 0, 1),
        );
        let edge = Dependency {
            dependent: String::from("app"),
            dependency: String::from("probe"),
        };

        let built = fleet(Path::new("/fleet"), vec![repo], vec![edge]);

        assert_eq!(built.root, Path::new("/fleet"));
        assert_eq!(built.repos.len(), 1);
        assert_eq!(built.dependencies.len(), 1);
    }
}
