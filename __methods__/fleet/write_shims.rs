use std::os::unix::fs::PermissionsExt;

use crate::Error;
use crate::basic;
use crate::fleet::Fleet;

pub fn write_shims(fleet: &Fleet) -> Result<(), Error> {
    let shimmed_tools = [
        basic::structs::ShimmedTool {
            tool: "lele-lint",
            repo: "lele-rust-linter",
            package: "lele-lint",
        },
        basic::structs::ShimmedTool {
            tool: "lele-bevy-lint",
            repo: "lele-rust-linter",
            package: "lele-bevy-lint",
        },
        basic::structs::ShimmedTool {
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

#[cfg(test)]
mod tests {
    use super::write_shims;

    #[test]
    fn test_usage() {
        let _ = write_shims;
    }
}
