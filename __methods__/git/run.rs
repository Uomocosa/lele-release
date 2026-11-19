use std::path::Path;

use crate::Error;
use crate::git::Git;

pub fn run(dir: &Path, args: &[&str]) -> Result<(), Error> {
    let command = Git::command_without_repo_environment(dir, args)
        .output()
        .map_err(|error| Error::Io {
            context: format!("cannot run git in {}", dir.display()),
            source: error,
        })?;
    if command.status.success() {
        return Ok(());
    }
    Err(Error::Git {
        repo: dir.display().to_string(),
        command: args.join(" "),
        stderr: String::from_utf8_lossy(&command.stderr).into_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::run;

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().unwrap();

        let initialised = run(dir.path(), &["init", "-q", "-b", "main"]);
        let missing_head = run(dir.path(), &["rev-parse", "HEAD"]);

        assert!(initialised.is_ok());
        assert!(missing_head.is_err());
    }
}
