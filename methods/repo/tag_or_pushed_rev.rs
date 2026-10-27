use crate::Error;
use crate::git::Git;
use crate::repo::Repo;

pub fn tag_or_pushed_rev(dependency: &Repo) -> Result<(bool, String), Error> {
    if let Some(tag) = dependency.latest_tag()? {
        return Ok((true, tag));
    }
    let head = Git::output(&dependency.dir, &["rev-parse", "HEAD"])?;
    let pushed = Git::output(&dependency.dir, &["rev-parse", "origin/main"]).unwrap_or(head);
    Ok((false, pushed))
}

#[cfg(test)]
mod tests {
    use super::tag_or_pushed_rev;

    #[test]
    fn test_usage() {
        let _ = tag_or_pushed_rev;
    }
}
