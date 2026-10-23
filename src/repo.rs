use std::path::PathBuf;

use atomic_delegate_macros::atomic_delegates;
use semver::Version;

use crate::__basic__;
use crate::Error;
use crate::fleet::Fleet;

pub struct Repo {
    pub name: String,
    pub dir: PathBuf,
    pub url: String,
    pub packages: Vec<__basic__::structs::Package>,
    pub manifests: Vec<PathBuf>,
    pub version_manifest: PathBuf,
    pub version: Version,
}

#[rustfmt::skip]
impl Repo {
    pub fn manifests(&self) -> Result<Vec<PathBuf>, Error> { Ok(self.manifests.clone()) }
}

#[atomic_delegates]
impl Repo {
    #[must_use]
    pub fn latest_tag(&self) -> Result<Option<String>, Error> {}
    #[must_use]
    pub fn commits_since_tag(&self) -> Result<Option<u64>, Error> {}
    #[must_use]
    pub fn matches_url(&self, url: &str) -> bool {}
    pub fn ensure_ready(&self, fleet: &Fleet) -> Result<(), Error> {}
    pub fn release(&self, next: &Version) -> Result<(), Error> {}
}

#[cfg(test)]
mod tests {
    use super::Repo;

    #[test]
    fn test_usage() {
        let _ = Repo::latest_tag;
        let _ = Repo::commits_since_tag;
        let _ = Repo::matches_url;
        let _ = Repo::ensure_ready;
        let _ = Repo::release;
        let _ = Repo::manifests;
    }
}
