use std::collections::HashMap;
use std::fmt::Write as _;
use std::path::Path;

use crate::Error;
use crate::fleet::Fleet;
use crate::git::Git;
use crate::repo::Repo;

pub fn clean_lock(fleet: &Fleet, lock: &Path, input: &str) -> Result<String, Error> {
    let Some(current) = fleet.repos.iter().find(|repo| lock.starts_with(&repo.dir)) else {
        return Ok(input.to_string());
    };
    let mut sources: HashMap<String, (String, String)> = HashMap::new();
    for repo in fleet.repos.iter().filter(|repo| repo.name != current.name) {
        let Some(tag) = repo.latest_tag()? else {
            continue;
        };
        let sha = Git::output(&repo.dir, &["rev-parse", &format!("{tag}^{{commit}}")])?;
        let version = tag.trim_start_matches('v').to_string();
        let source = format!("git+{}?tag={tag}#{sha}", repo.url.trim_end_matches(".git"));
        for package in &repo.packages {
            sources.insert(package.name.clone(), (version.clone(), source.clone()));
        }
    }
    Ok(rewrite(input, current, &sources))
}

// needed helper: put every fleet entry in the committed (git) form, key order preserved
fn rewrite(input: &str, current: &Repo, sources: &HashMap<String, (String, String)>) -> String {
    let mut out = String::new();
    let mut block = String::new();
    for line in input.lines() {
        if line == "[[package]]" && !block.is_empty() {
            out.push_str(&rewrite_block(&block, current, sources));
            block.clear();
        }
        block.push_str(line);
        block.push('\n');
    }
    if !block.is_empty() {
        out.push_str(&rewrite_block(&block, current, sources));
    }
    out
}

// needed helper: one `[[package]]` block
fn rewrite_block(
    block: &str,
    current: &Repo,
    sources: &HashMap<String, (String, String)>,
) -> String {
    let Some(name) = block
        .lines()
        .find_map(|line| line.strip_prefix("name = \"")?.strip_suffix('"'))
    else {
        return block.to_string();
    };
    let desired = if current.packages.iter().any(|package| package.name == name) {
        Some((current.version.to_string(), None))
    } else {
        sources
            .get(name)
            .map(|(version, source)| (version.clone(), Some(source.clone())))
    };
    let Some((version, source)) = desired else {
        return block.to_string();
    };
    let mut out = String::new();
    for line in block.lines() {
        if line.starts_with("source = ") {
            continue;
        }
        if line.starts_with("version = ") {
            let _ = writeln!(out, "version = \"{version}\"");
            if let Some(source) = &source {
                let _ = writeln!(out, "source = \"{source}\"");
            }
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::clean_lock;

    #[test]
    fn test_usage() {
        let _ = clean_lock;
    }
}
