use crate::fleet::Fleet;

#[must_use]
pub fn without_fleet_lines(fleet: &Fleet, text: &str) -> String {
    let names: Vec<String> = fleet
        .repos
        .iter()
        .flat_map(|repo| repo.packages.iter().map(|package| package.name.clone()))
        .collect();
    text.lines()
        .filter(|line| {
            let trimmed = line.trim_start();
            !names
                .iter()
                .any(|name| trimmed.starts_with(&format!("{name} = ")))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use semver::Version;

    use super::without_fleet_lines;
    use crate::basic;
    use crate::fleet::Fleet;
    use crate::repo::Repo;

    #[test]
    fn test_usage() {
        let dir = std::env::temp_dir();
        let fleet = Fleet {
            root: dir.clone(),
            repos: vec![Repo {
                name: String::from("dep"),
                dir: dir.clone(),
                url: String::new(),
                packages: vec![basic::structs::Package {
                    name: String::from("dep"),
                    manifest: dir.join("Cargo.toml"),
                }],
                manifests: Vec::new(),
                version_manifest: dir.join("Cargo.toml"),
                version: Version::new(0, 0, 1),
            }],
            dependencies: Vec::new(),
        };
        let text = "[dependencies]\n  dep = { path = \"../dep\" }\nserde = \"1\"";
        assert_eq!(
            without_fleet_lines(&fleet, text),
            "[dependencies]\nserde = \"1\""
        );
    }
}
