use std::path::PathBuf;

use crate::Error;
use crate::fleet::Fleet;

pub fn setup(fleet: &Fleet) -> Result<(), Error> {
    let executable = std::env::current_exe().map_err(|error| Error::Io {
        context: String::from("cannot locate this binary"),
        source: error,
    })?;
    let Some(home) = std::env::var_os("HOME").map(PathBuf::from) else {
        return Err(Error::Refused(String::from("HOME is not set")));
    };
    let config = home.join(".config/git/lele-pins.gitconfig");
    if let Some(parent) = config.parent() {
        std::fs::create_dir_all(parent).map_err(|error| Error::Io {
            context: format!("{}", parent.display()),
            source: error,
        })?;
    }
    let binary = executable.display();
    let template = include_str!("../../pins.gitconfig.template");
    let definition = template.replace("@BINARY@", &binary.to_string());
    std::fs::write(&config, definition).map_err(|error| Error::Io {
        context: format!("{}", config.display()),
        source: error,
    })?;
    Fleet::include_pointer(&home)?;
    fleet.write_shims()?;
    println!("filter definition: {}", config.display());
    println!("binary:            {binary}");
    println!();
    for repo in &fleet.repos {
        let attributes =
            std::fs::read_to_string(repo.dir.join(".gitattributes")).unwrap_or_default();
        let present = attributes
            .lines()
            .any(|line| line.contains("filter=lele-pins"));
        println!(
            "  {:<28} .gitattributes {}",
            repo.name,
            if present {
                "ok"
            } else {
                "MISSING `Cargo.toml filter=lele-pins`"
            }
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::process::Command;

    use super::setup;
    use crate::testing;

    const CHILD_MARKER: &str = "LELE_RELEASE_TEST_SETUP_HOME";

    #[test]
    fn test_usage() {
        let Ok(home) = std::env::var(CHILD_MARKER) else {
            let tmp = tempfile::tempdir().unwrap();
            let child = Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "methods::fleet::setup::tests::test_usage",
                    "--nocapture",
                ])
                .env(CHILD_MARKER, tmp.path())
                .env("HOME", tmp.path())
                .output()
                .unwrap();
            let stdout = String::from_utf8_lossy(&child.stdout);
            assert!(child.status.success(), "{stdout}");
            assert!(
                tmp.path().join(".config/git/lele-pins.gitconfig").is_file(),
                "{stdout}"
            );
            assert!(tmp.path().join("bin/lele-lint").is_file(), "{stdout}");
            return;
        };
        let fleet = testing::fleet(Path::new(&home), Vec::new(), Vec::new());

        let result = setup(&fleet);

        assert!(result.is_ok());
    }
}
