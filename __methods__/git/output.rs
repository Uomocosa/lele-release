use std::path::Path;
use std::process::Command;

use crate::Error;

pub fn output(dir: &Path, args: &[&str]) -> Result<String, Error> {
    let command = Command::new("git").args(args).current_dir(dir).output();
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
