use std::path::{Path, PathBuf};

use atomic_delegate_macros::atomic_delegates;

use crate::__basic__;
use crate::Error;
use crate::repo::Repo;

pub struct Fleet {
    pub root: PathBuf,
    pub repos: Vec<Repo>,
    pub dependencies: Vec<__basic__::structs::Dependency>,
}

#[atomic_delegates]
impl Fleet {
    pub fn status(&self) -> Result<(), Error> {}
    #[must_use]
    pub fn plan(
        &self,
        level: __basic__::enums::Bump,
    ) -> Result<Vec<__basic__::structs::ReleaseStep>, Error> {
    }
    pub fn release(
        &self,
        level: __basic__::enums::Bump,
        selection: &__basic__::enums::Selection,
        dry_run: bool,
    ) -> Result<(), Error> {
    }
    pub fn setup(&self) -> Result<(), Error> {}
    pub fn pins(&self, repo: &Repo) -> Result<Vec<__basic__::structs::Pin>, Error> {}
    pub fn clean_manifest(&self, manifest: &Path, input: &str) -> Result<String, Error> {}
    pub fn smudge_manifest(&self, manifest: &Path, input: &str) -> Result<String, Error> {}
    pub fn clean_lock(&self, lock: &Path, input: &str) -> Result<String, Error> {}
    pub fn smudge_lock(&self, lock: &Path, input: &str) -> Result<String, Error> {}
}

#[atomic_delegates]
impl TryFrom<&Path> for Fleet {
    type Error = Error;
    fn try_from(root: &Path) -> Result<Self, Error> {}
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::Fleet;

    #[test]
    fn test_usage() {
        let _ = <Fleet as TryFrom<&Path>>::try_from;
        let _ = Fleet::plan;
        let _ = Fleet::release;
        let _ = Fleet::setup;
        let _ = Fleet::pins;
        let _ = Fleet::clean_manifest;
        let _ = Fleet::smudge_manifest;
        let _ = Fleet::clean_lock;
        let _ = Fleet::smudge_lock;
    }
}
