use crate::Error;
use crate::git::Git;
use crate::repo::Repo;

pub fn stage_version_files(repo: &Repo) -> Result<(), Error> {
    let mut paths: Vec<String> = Vec::new();
    for manifest in &repo.manifests {
        let Ok(relative) = manifest.strip_prefix(&repo.dir) else {
            return Err(Error::Refused(format!(
                "{}: manifest is outside the repo",
                repo.name
            )));
        };
        paths.push(relative.display().to_string());
    }
    if repo.dir.join("Cargo.lock").exists() {
        paths.push(String::from("Cargo.lock"));
    }
    let mut args = vec!["add", "--"];
    args.extend(paths.iter().map(String::as_str));
    Git::run(&repo.dir, &args)
}

#[cfg(test)]
mod tests {
    use super::stage_version_files;

    #[test]
    fn test_usage() {
        let _ = stage_version_files;
    }
}
