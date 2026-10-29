use crate::Error;
use crate::repo::Repo;

pub fn install(repo: &Repo) -> Result<(), Error> {
    let output = std::process::Command::new("just")
        .arg("install")
        .current_dir(&repo.dir)
        .output()
        .map_err(|error| Error::Io {
            context: format!("{}: cannot run `just install`", repo.name),
            source: error,
        })?;
    if !output.status.success() {
        return Err(Error::Refused(format!(
            "{}: `just install` failed\n{}",
            repo.name,
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::install;

    #[test]
    fn test_usage() {
        let _ = install;
    }
}
