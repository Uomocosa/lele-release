use std::path::Path;

use crate::Error;
use crate::git::Git;

pub fn output(dir: &Path, args: &[&str]) -> Result<String, Error> {
    let command = Git::command_without_repo_environment(dir, args).output();
    let output = command.map_err(|error| Error::Io {
        context: format!("cannot run git in {}", dir.display()),
        source: error,
    })?;
    if !output.status.success() {
        return Err(Error::Git {
            repo: dir.display().to_string(),
            command: args.join(" "),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        });
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::output;

    #[test]
    fn test_usage() {
        let _ = output;
    }
}
