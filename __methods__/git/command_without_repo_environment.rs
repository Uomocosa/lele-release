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
    use std::path::Path;

    use super::command_without_repo_environment;

    #[test]
    fn test_usage() {
        let _ = command_without_repo_environment(Path::new("."), &["status"]);
    }
}
