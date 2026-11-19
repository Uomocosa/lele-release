use std::path::Path;

use crate::Git;

pub fn commit_tag(dir: &Path, message: &str, tag: &str) {
    Git::run(dir, &["config", "user.name", "t"]).unwrap();
    Git::run(dir, &["config", "user.email", "t@t"]).unwrap();
    Git::run(
        dir,
        &[
            "-c",
            "user.name=t",
            "-c",
            "user.email=t@t",
            "commit",
            "-qm",
            message,
        ],
    )
    .unwrap();
    Git::run(dir, &["tag", "-a", tag, "-m", tag]).unwrap();
}

#[cfg(test)]
mod tests {
    use super::commit_tag;
    use crate::Git;

    #[test]
    fn test_usage() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        std::fs::write(
            dir.join("Cargo.toml"),
            "[package]\nname = \"probe\"\nversion = \"0.0.1\"\n",
        )
        .unwrap();
        Git::run(dir, &["init", "-q", "-b", "main"]).unwrap();
        Git::run(dir, &["add", "--", "Cargo.toml"]).unwrap();

        commit_tag(dir, "v0.0.1", "v0.0.1");

        let tags = Git::output(dir, &["tag", "--list"]).unwrap();
        assert!(tags.contains("v0.0.1"), "{tags}");
    }
}
