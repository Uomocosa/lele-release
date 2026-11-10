use crate::repo::Repo;

#[must_use]
pub fn matches_url(repo: &Repo, url: &str) -> bool {
    let left = Repo::slug(&repo.url);
    let right = Repo::slug(url);
    !left.is_empty() && left == right
}

#[cfg(test)]
mod tests {
    use super::matches_url;

    #[test]
    fn test_usage() {
        let _ = matches_url;
    }
}
