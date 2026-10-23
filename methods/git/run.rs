use std::path::Path;

use crate::Error;

pub fn run(dir: &Path, args: &[&str]) -> Result<(), Error> {
    let command = std::process::Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .map_err(|error| Error::Io {
            message: format!("cannot run git in {}: {error}", dir.display()),
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
        let _ = run;
    }
}
