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
            Ok(repo) => fresh.relink_tag_pins(repo, false).and_then(|relinked| {
                Fleet::print_relinked_pins(&relinked);
                Repo::release(repo, &step.next)
            }),
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
    use crate::testing;

    #[test]
    fn test_usage() {
        let tmp = tempfile::tempdir().unwrap();
        let fleet = testing::fleet(&tmp.path().join("missing"), Vec::new(), Vec::new());
        let steps = vec![testing::release_step("probe")];

        let (completed, error) = run_release_loop(&fleet, &steps);

        assert!(completed.is_empty());
        assert!(error.is_some());
    }
}
