use std::path::PathBuf;

use semver::Version;

use crate::__basic__;

#[derive(Clone, Debug)]
pub struct Package {
    pub name: String,
    pub manifest: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dependency {
    pub dependent: String,
    pub dependency: String,
}

#[derive(Clone, Debug)]
pub struct Pin {
    pub manifest: PathBuf,
    pub dependent: String,
    pub dependency: String,
    pub form: __basic__::enums::PinForm,
}

#[derive(Clone, Debug)]
pub struct ReleaseStep {
    pub repo: String,
    pub current: Version,
    pub next: Version,
    pub reason: String,
}
