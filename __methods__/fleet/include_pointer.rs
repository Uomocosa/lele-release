use std::path::Path;

use crate::Error;

pub fn include_pointer(home: &Path) -> Result<(), Error> {
    let gitconfig = home.join(".gitconfig");
    let existing = std::fs::read_to_string(&gitconfig).unwrap_or_default();
    if existing.contains("lele-pins.gitconfig") {
        println!("includeIf already present in {}", gitconfig.display());
        return Ok(());
    }
    let block = "\n[includeIf \"hasconfig:remote.*.url:https://github.com/Uomocosa/**\"]\n    path = ~/.config/git/lele-pins.gitconfig\n";
    std::fs::write(&gitconfig, format!("{existing}{block}")).map_err(|error| Error::Io {
        context: format!("{}", gitconfig.display()),
        source: error,
    })?;
    println!("added includeIf to {}", gitconfig.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::include_pointer;

    #[test]
    fn test_usage() {
        let home = tempfile::tempdir().unwrap();

        include_pointer(home.path()).unwrap();

        let first = std::fs::read_to_string(home.path().join(".gitconfig")).unwrap();
        assert!(first.contains("lele-pins.gitconfig"), "{first}");

        include_pointer(home.path()).unwrap();

        let repeated = std::fs::read_to_string(home.path().join(".gitconfig")).unwrap();
        assert_eq!(
            repeated.matches("lele-pins.gitconfig").count(),
            1,
            "{repeated}"
        );
    }
}
