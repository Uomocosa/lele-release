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
        (basic::enums::PinsMode::Smudge, true) => Ok(fleet.smudge_lock(&manifest, &input)),
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
    use std::io::Write as _;
    use std::process::Command;
    use std::process::Stdio;

    use super::filter_stdin_or_pass_through;
    use crate::PinsMode;

    const CHILD_MARKER: &str = "LELE_RELEASE_TEST_FILTER_STDIN";

    #[test]
    fn test_usage() {
        let Ok(_marker) = std::env::var(CHILD_MARKER) else {
            let input = "[package]\nname = \"probe\"\nversion = \"0.0.1\"\n";
            let mut child = Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "methods::fleet::filter_stdin_or_pass_through::tests::test_usage",
                    "--nocapture",
                ])
                .env(CHILD_MARKER, "1")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .unwrap();
            child
                .stdin
                .as_mut()
                .unwrap()
                .write_all(input.as_bytes())
                .unwrap();
            let output = child.wait_with_output().unwrap();
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(output.status.success(), "{stdout}");
            assert!(stdout.contains("name = \"probe\""), "{stdout}");
            return;
        };
        let root = tempfile::tempdir().unwrap();

        let result = filter_stdin_or_pass_through(PinsMode::Smudge, None, root.path());

        assert!(result.is_ok());
    }
}
