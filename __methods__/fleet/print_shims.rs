use crate::fleet::Fleet;

pub fn print_shims(fleet: &Fleet) {
    let bin = fleet.root.join("bin");
    let Ok(entries) = std::fs::read_dir(&bin) else {
        println!(
            "shims   none in {} (run `lele-release setup`)",
            bin.display()
        );
        return;
    };
    let path = std::env::var_os("PATH").unwrap_or_default();
    let mut tools: Vec<String> = entries
        .filter_map(Result::ok)
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect();
    tools.sort();
    for tool in tools {
        let resolved = std::env::split_paths(&path)
            .map(|dir| dir.join(&tool))
            .find(|candidate| candidate.is_file());
        let state = match resolved {
            Some(found) if found.parent() == Some(bin.as_path()) => {
                String::from("sibling build (shim active)")
            }
            Some(found) => format!(
                "{} (shim shadowed: put {} first on PATH)",
                found.display(),
                bin.display()
            ),
            None => String::from("not on PATH"),
        };
        println!("shims   {tool:<28} {state}");
    }
}

#[cfg(test)]
mod tests {
    use super::print_shims;

    #[test]
    fn test_usage() {
        let _ = print_shims;
    }
}
