use std::path::PathBuf;

use crate::Error;
use crate::fleet::Fleet;

pub fn setup(fleet: &Fleet) -> Result<(), Error> {
    let executable = std::env::current_exe().map_err(|error| Error::Io {
        message: format!("cannot locate this binary: {error}"),
    })?;
    let Some(home) = std::env::var_os("HOME").map(PathBuf::from) else {
        return Err(Error::Refused(String::from("HOME is not set")));
    };
    let config = home.join(".config/git/lele-pins.gitconfig");
    if let Some(parent) = config.parent() {
        std::fs::create_dir_all(parent).map_err(|error| Error::Io {
            message: format!("{}: {error}", parent.display()),
        })?;
    }
    let binary = executable.display();
    let template = include_str!("../../pins.gitconfig.template");
    let definition = template.replace("@BINARY@", &binary.to_string());
    std::fs::write(&config, definition).map_err(|error| Error::Io {
        message: format!("{}: {error}", config.display()),
    })?;
    include_pointer(&home)?;
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

// needed helper: one includeIf pointer in the global git config (idempotent)
fn include_pointer(home: &std::path::Path) -> Result<(), Error> {
    let gitconfig = home.join(".gitconfig");
    let existing = std::fs::read_to_string(&gitconfig).unwrap_or_default();
    if existing.contains("lele-pins.gitconfig") {
        println!("includeIf already present in {}", gitconfig.display());
        return Ok(());
    }
    let block = "\n[includeIf \"hasconfig:remote.*.url:https://github.com/Uomocosa/**\"]\n    path = ~/.config/git/lele-pins.gitconfig\n";
    std::fs::write(&gitconfig, format!("{existing}{block}")).map_err(|error| Error::Io {
        message: format!("{}: {error}", gitconfig.display()),
    })?;
    println!("added includeIf to {}", gitconfig.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::setup;

    #[test]
    fn test_usage() {
        let _ = setup;
    }
}
