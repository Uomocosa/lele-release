#[must_use]
pub fn slug(url: &str) -> String {
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
    use super::slug;

    #[test]
    fn test_usage() {
        let ssh = slug("git@github.com:Uomocosa/lele-release.git");
        let https = slug("https://github.com/uomocosa/lele-release/");
        assert_eq!(ssh, "uomocosa/lele-release");
        assert_eq!(ssh, https);
    }
}
