use std::io::Read;
use std::path::Path;

use crate::Error;
use crate::basic;
use crate::fleet::Fleet;

pub fn filter_stdin_or_pass_through(
    mode: basic::enums::PinsMode,
    path: Option<&Path>,
    root: &Path,
) -> Result<(), Error> {
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .map_err(|error| Error::Io {
            context: String::from("stdin"),
            source: error,
        })?;
    let Some(path) = path else {
        print!("{input}");
        return Ok(());
    };
    let Ok(cwd) = std::env::current_dir() else {
        print!("{input}");
        return Ok(());
    };
    let manifest = cwd.join(path);
    let Ok(fleet) = Fleet::try_from(root) else {
        print!("{input}");
        return Ok(());
    };
    let is_lock = path.file_name().and_then(|name| name.to_str()) == Some("Cargo.lock");
    let output = match (mode, is_lock) {
        (basic::enums::PinsMode::Clean, true) => fleet.clean_lock(&manifest, &input),
        (basic::enums::PinsMode::Smudge, true) => fleet.smudge_lock(&manifest, &input),
        (basic::enums::PinsMode::Clean, false) => fleet.clean_manifest(&manifest, &input),
        (basic::enums::PinsMode::Smudge, false) => fleet.smudge_manifest(&manifest, &input),
    };
    match output {
        Ok(text) => print!("{text}"),
        Err(_) => print!("{input}"),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::filter_stdin_or_pass_through;

    #[test]
    fn test_usage() {
        let _ = filter_stdin_or_pass_through;
    }
}
