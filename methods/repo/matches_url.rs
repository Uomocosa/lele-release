use crate::repo::Repo;

#[must_use]
pub fn matches_url(repo: &Repo, url: &str) -> bool {
    let left = slug(&repo.url);
    let right = slug(url);
    !left.is_empty() && left == right
}

// needed helper: `owner/repo` so ssh and https remotes compare equal
fn slug(url: &str) -> String {
    let cleaned = url.trim().trim_end_matches(".git").trim_end_matches('/');
    let normalised = cleaned.replace(':', "/");
    let parts: Vec<&str> = normalised.rsplit('/').take(2).collect();
    let (Some(owner), Some(name)) = (parts.get(1), parts.first()) else {
        return cleaned.to_ascii_lowercase();
    };
    format!(
        "{}/{}",
        owner.to_ascii_lowercase(),
        name.to_ascii_lowercase()
    )
}

#[cfg(test)]
mod tests {
    use super::matches_url;

    #[test]
    fn test_usage() {
        let _ = matches_url;
    }
}
