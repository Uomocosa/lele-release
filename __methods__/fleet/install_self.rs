use crate::Error;
use crate::basic;
use crate::fleet::Fleet;

#[must_use]
pub fn install_self(
    fleet: &Fleet,
    completed: &[basic::structs::ReleaseStep],
    no_install: bool,
) -> (bool, Option<Error>) {
    if !Fleet::self_install_due(completed, no_install) {
        return (false, None);
    }
    let result = match Fleet::try_from(fleet.root.as_path()) {
        Ok(fresh) => match fresh.repo_named("lele-release") {
            Ok(repo) => repo.install(),
            Err(error) => Err(error),
        },
        Err(error) => Err(error),
    };
    match result {
        Ok(()) => {
            println!("install ✓ lele-release (just install)");
            (true, None)
        }
        Err(error) => {
            eprintln!("install failed: run `just install` in lele-release");
            (false, Some(error))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::install_self;

    #[test]
    fn test_usage() {
        let _ = install_self;
    }
}
