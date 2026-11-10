use crate::Error;
use crate::basic;
use crate::fleet::Fleet;
use crate::repo::Repo;

#[must_use]
pub fn run_release_loop(
    fleet: &Fleet,
    steps: &[basic::structs::ReleaseStep],
) -> (Vec<basic::structs::ReleaseStep>, Option<Error>) {
    let mut completed: Vec<basic::structs::ReleaseStep> = Vec::new();
    for step in steps {
        let fresh = match Fleet::try_from(fleet.root.as_path()) {
            Ok(fresh) => fresh,
            Err(error) => return (completed, Some(error)),
        };
        let outcome = match fresh.repo_named(&step.repo) {
            Ok(repo) => Repo::release(repo, &step.next),
            Err(error) => Err(error),
        };
        match outcome {
            Ok(()) => {
                println!(
                    "[{}] {:<28} {} · tag v{} · pushed",
                    step.repo, step.current, step.next, step.next
                );
                completed.push(step.clone());
            }
            Err(error) => return (completed, Some(error)),
        }
    }
    (completed, None)
}

#[cfg(test)]
mod tests {
    use super::run_release_loop;

    #[test]
    fn test_usage() {
        let _ = run_release_loop;
    }
}
