use crate::Error;
use crate::repo::Repo;

pub fn run_just_gate(repo: &Repo) -> Result<(), Error> {
    for recipe in ["fmt", "lint", "test"] {
        let output = std::process::Command::new("just")
            .arg(recipe)
            .current_dir(&repo.dir)
            .output()
            .map_err(|error| Error::Io {
                context: format!("{}: cannot run `just {recipe}`", repo.name),
                source: error,
            })?;
        if !output.status.success() {
            return Err(Error::Refused(format!(
                "{}: `just {recipe}` failed\n{}",
                repo.name,
                String::from_utf8_lossy(&output.stderr)
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::run_just_gate;

    #[test]
    fn test_usage() {
        let _ = run_just_gate;
    }
}
