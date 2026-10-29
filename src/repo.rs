use std::collections::HashMap;
use std::hash::BuildHasher;
use std::path::PathBuf;

use atomic_delegate_macros::atomic_delegates;
use semver::Version;

use crate::Error;
use crate::basic;
use crate::fleet::Fleet;
use crate::git::Git;

pub struct Repo {
    pub name: String,
    pub dir: PathBuf,
    pub url: String,
    pub packages: Vec<basic::structs::Package>,
    pub manifests: Vec<PathBuf>,
    pub version_manifest: PathBuf,
    pub version: Version,
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
    pub fn install(&self) -> Result<(), Error> {}
}

#[atomic_delegates]
impl Repo {
    pub(crate) fn slug(url: &str) -> String {}
    pub(crate) fn run_just_gate(&self) -> Result<(), Error> {}
    pub(crate) fn refuse_unclean_tree(&self, fleet: &Fleet) -> Result<(), Error> {}
    pub(crate) fn refuse_lock_drift(&self) -> Result<(), Error> {}
    pub(crate) fn untrimmed_status_paths(&self) -> Result<Vec<String>, Error> {}
    pub(crate) fn has_only_fleet_drift(
        &self,
        fleet: &Fleet,
        relative: &str,
    ) -> Result<bool, Error> {
    }
    pub(crate) fn commit_tag_and_push(&self, next: &Version, tag: &str) -> Result<(), Error> {}
    pub(crate) fn write_version(&self, next: &Version) -> Result<(), Error> {}
    pub(crate) fn verify_staged_lock(&self, next: &Version) -> Result<(), Error> {}
    pub(crate) fn bump_lock_version(&self, next: &Version) -> Result<(), Error> {}
    pub(crate) fn stage_version_files(&self) -> Result<(), Error> {}
    pub(crate) fn tag_or_pushed_rev(&self) -> Result<(bool, String), Error> {}
    pub(crate) fn tree_state(&self) -> String {}
    pub(crate) fn clean_lock_text<S: BuildHasher>(
        &self,
        input: &str,
        sources: &HashMap<String, (String, String), S>,
    ) -> String {
    }
    pub(crate) fn clean_lock_block<S: BuildHasher>(
        &self,
        block: &str,
        sources: &HashMap<String, (String, String), S>,
    ) -> String {
    }
    pub(crate) fn smudge_lock_text<S: BuildHasher>(
        &self,
        input: &str,
        versions: &HashMap<String, String, S>,
    ) -> String {
    }
    pub(crate) fn smudge_lock_block<S: BuildHasher>(
        &self,
        block: &str,
        versions: &HashMap<String, String, S>,
    ) -> String {
    }
}

#[rustfmt::skip]
impl Repo {
    pub(crate) fn rollback_to_origin(&self, tag: &str) {
        let _ = Git::run(&self.dir, &["reset", "--hard", "origin/main"]);
        let _ = Git::run(&self.dir, &["tag", "-d", tag]);
    }
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
    }
}
