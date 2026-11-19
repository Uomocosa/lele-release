use std::path::Path;
use std::process::Command;

use crate::constants;

pub fn command_without_repo_environment(dir: &Path, args: &[&str]) -> Command {
    let mut command = Command::new("git");
    command.args(args).current_dir(dir);
    for name in constants::GIT_REPO_LOCATING_ENV_VAR_NAMES {
        command.env_remove(name);
    }
    command
}

#[cfg(test)]
mod tests {
    use super::command_without_repo_environment;

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().unwrap();

        let command = command_without_repo_environment(dir.path(), &["status"]);

        assert_eq!(command.get_current_dir(), Some(dir.path()));
        assert_eq!(
            command
                .get_args()
                .map(|argument| argument.to_str().unwrap())
                .collect::<Vec<_>>(),
            vec!["status"]
        );
        assert!(
            command
                .get_envs()
                .any(|(name, value)| name.to_string_lossy() == "GIT_DIR" && value.is_none())
        );
    }
}
