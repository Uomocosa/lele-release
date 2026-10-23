use std::os::unix::fs::PermissionsExt;
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
    include_pointer(&home)?;
    write_shims(fleet)?;
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

// needed helper: the sibling repo and package a shimmed tool is built from
struct ShimmedTool {
    tool: &'static str,
    repo: &'static str,
    package: &'static str,
}

// needed helper: one shim per fleet tool in `<fleet>/bin`, built from the sibling checkout
fn write_shims(fleet: &Fleet) -> Result<(), Error> {
    let shimmed_tools = [
        ShimmedTool {
            tool: "lele-lint",
            repo: "lele-rust-linter",
            package: "lele-lint",
        },
        ShimmedTool {
            tool: "lele-bevy-lint",
            repo: "lele-rust-linter",
            package: "lele-bevy-lint",
        },
        ShimmedTool {
            tool: "lele-function-taxonomy",
            repo: "lele-rust-function-taxonomy",
            package: "lele-rust-function-taxonomy",
        },
    ];
    let bin = fleet.root.join("bin");
    std::fs::create_dir_all(&bin).map_err(|error| Error::Io {
        context: format!("{}", bin.display()),
        source: error,
    })?;
    let template = include_str!("../../shim.sh.template");
    for shimmed in &shimmed_tools {
        let repo = fleet.root.join(shimmed.repo);
        let quoted_repo = repo.display().to_string().replace('\'', r"'\''");
        let script = template
            .replace("@TOOL@", shimmed.tool)
            .replace("@PACKAGE@", shimmed.package)
            .replace("@REPO@", &quoted_repo);
        let shim = bin.join(shimmed.tool);
        std::fs::write(&shim, script)
            .and_then(|()| std::fs::set_permissions(&shim, std::fs::Permissions::from_mode(0o755)))
            .map_err(|error| Error::Io {
                context: format!("{}", shim.display()),
                source: error,
            })?;
        println!(
            "shim:              {} -> {}",
            shim.display(),
            repo.display()
        );
    }
    println!(
        "put the shims before ~/.cargo/bin in your shell profile, then check `lele-release status`:"
    );
    println!("  export PATH=\"{}:$PATH\"", bin.display());
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
        context: format!("{}", gitconfig.display()),
        source: error,
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
