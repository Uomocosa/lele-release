use std::path::{Path, PathBuf};

use atomic_delegate_macros::atomic_delegates;
use semver::Version;
use toml_edit::{DocumentMut, Item, TableLike};

use crate::Error;
use crate::basic;
use crate::repo::Repo;

pub struct Fleet {
    pub root: PathBuf,
    pub repos: Vec<Repo>,
    pub dependencies: Vec<basic::structs::Dependency>,
}

#[atomic_delegates]
impl TryFrom<&Path> for Fleet {
    type Error = Error;
    fn try_from(root: &Path) -> Result<Self, Error> {}
}

#[atomic_delegates]
impl Fleet {
    pub fn status(&self) -> Result<(), Error> {}
    #[must_use]
    pub fn plan(
        &self,
        level: basic::enums::Bump,
    ) -> Result<Vec<basic::structs::ReleaseStep>, Error> {
    }
    pub fn release(
        &self,
        level: basic::enums::Bump,
        selection: &basic::enums::Selection,
        dry_run: bool,
    ) -> Result<(), Error> {
    }
    pub fn setup(&self) -> Result<(), Error> {}
    pub fn pins(&self, repo: &Repo) -> Result<Vec<basic::structs::Pin>, Error> {}
    pub fn clean_manifest(&self, manifest: &Path, input: &str) -> Result<String, Error> {}
    pub fn smudge_manifest(&self, manifest: &Path, input: &str) -> Result<String, Error> {}
    pub fn clean_lock(&self, lock: &Path, input: &str) -> Result<String, Error> {}
    pub fn smudge_lock(&self, lock: &Path, input: &str) -> Result<String, Error> {}
    pub fn filter_stdin_or_pass_through(
        mode: basic::enums::PinsMode,
        path: Option<&Path>,
        root: &Path,
    ) -> Result<(), Error> {
    }
}

#[atomic_delegates]
impl Fleet {
    pub(crate) fn without_fleet_lines(&self, text: &str) -> String {}
    pub(crate) fn clean_manifest_section(
        &self,
        document: &mut DocumentMut,
        section: &str,
        base: &Path,
        current: &str,
    ) -> Result<bool, Error> {
    }
    pub(crate) fn smudge_manifest_section(
        &self,
        document: &mut DocumentMut,
        section: &str,
        base: &Path,
    ) -> bool {
    }
    pub(crate) fn normalise_lexically(path: &Path) -> PathBuf {}
    pub(crate) fn dependency_items(document: &DocumentMut) -> Vec<(String, &Item)> {}
    pub(crate) fn pin_form(table: &dyn TableLike) -> Option<basic::enums::PinForm> {}
    pub(crate) fn changed_repos(&self) -> Result<Vec<(String, u64)>, Error> {}
    pub(crate) fn with_transitive_dependents(&self, changed: &[(String, u64)]) -> Vec<String> {}
    pub(crate) fn dependency_first_order(&self, affected: &[String]) -> Result<Vec<String>, Error> {
    }
    pub(crate) fn write_shims(&self) -> Result<(), Error> {}
    pub(crate) fn include_pointer(home: &Path) -> Result<(), Error> {}
    pub(crate) fn print_repos(&self) -> Result<(), Error> {}
    pub(crate) fn print_pins(&self) -> Result<(), Error> {}
    pub(crate) fn pin_state(&self, pin: &basic::structs::Pin) -> String {}
    pub(crate) fn print_shims(&self) {}
    pub(crate) fn print_filter(&self) {}
    pub(crate) fn print_plan(&self) -> Result<(), Error> {}
    pub(crate) fn discover_repo(dir: &Path) -> Result<Option<Repo>, Error> {}
    pub(crate) fn package_manifests(dir: &Path) -> Result<Vec<PathBuf>, Error> {}
    pub(crate) fn workspace_version(document: &DocumentMut) -> Option<Version> {}
    pub(crate) fn package_version(
        packages: &[basic::structs::Package],
        repo: &str,
    ) -> Result<(PathBuf, Version), Error> {
    }
    pub(crate) fn dependency_edges(repos: &[Repo]) -> Vec<basic::structs::Dependency> {}
    pub(crate) fn dependency_targets<'a>(
        repos: &'a [Repo],
        base: &Path,
        document: &DocumentMut,
    ) -> Vec<&'a Repo> {
    }
}

#[rustfmt::skip]
impl Fleet {
    pub(crate) fn read_manifest(manifest: &Path) -> Result<DocumentMut, Error> {
        let text = std::fs::read_to_string(manifest).map_err(|error| Self::io_error(manifest, error))?;
        text.parse::<DocumentMut>().map_err(|error| Error::Toml { path: manifest.to_path_buf(), source: error })
    }
    pub(crate) fn io_error(path: &Path, source: std::io::Error) -> Error {
        Error::Io { context: path.display().to_string(), source }
    }
    pub(crate) fn resolve_repo<'a>(repos: &'a [Repo], base: &Path, path: &str) -> Option<&'a Repo> {
        let target = Self::normalise_lexically(&base.join(path));
        repos.iter().find(|repo| {
            repo.packages.iter().any(|package| {
                package.manifest.parent().is_some_and(|dir| Self::normalise_lexically(dir) == target)
            })
        })
    }
    pub(crate) fn dependency_table_mut<'a>(
        document: &'a mut DocumentMut,
        section: &str,
    ) -> Option<&'a mut dyn TableLike> {
        if section == "workspace.dependencies" {
            return document
                .get_mut("workspace")?
                .as_table_like_mut()?
                .get_mut("dependencies")?
                .as_table_like_mut();
        }
        document.get_mut(section)?.as_table_like_mut()
    }
    pub(crate) fn points_at_fleet(
        &self,
        repo: &Repo,
        manifest: &Path,
        table: &dyn TableLike,
        form: &basic::enums::PinForm,
    ) -> bool {
        match form {
            basic::enums::PinForm::Path(path) => {
                let base = manifest.parent().unwrap_or_else(|| Path::new("."));
                Self::resolve_repo(&self.repos, base, path).is_some_and(|target| target.name != repo.name)
            }
            _ => table.get("git").and_then(Item::as_str).is_some_and(|url| {
                self.repos.iter().any(|candidate| candidate.matches_url(url))
            }),
        }
    }
    pub(crate) fn select_steps(
        &self,
        steps: Vec<basic::structs::ReleaseStep>,
        selection: &basic::enums::Selection,
    ) -> Result<Vec<basic::structs::ReleaseStep>, Error> {
        match selection {
            basic::enums::Selection::All => Ok(steps),
            basic::enums::Selection::Only(names) => {
                for name in names {
                    if !self.repos.iter().any(|repo| &repo.name == name) {
                        return Err(Error::Refused(format!("unknown repo `{name}`")));
                    }
                }
                Ok(steps.into_iter().filter(|step| names.contains(&step.repo)).collect())
            }
            basic::enums::Selection::Exclude(names) => {
                Ok(steps.into_iter().filter(|step| !names.contains(&step.repo)).collect())
            }
        }
    }
    pub(crate) fn repo_named(&self, name: &str) -> Result<&Repo, Error> {
        self.repos.iter().find(|repo| repo.name == name).ok_or_else(|| Error::Refused(format!("unknown repo `{name}`")))
    }
    pub(crate) const fn bumped(version: &Version, level: basic::enums::Bump) -> Version {
        match level {
            basic::enums::Bump::Patch => Version::new(version.major, version.minor, version.patch.saturating_add(1)),
            basic::enums::Bump::Minor => Version::new(version.major, version.minor.saturating_add(1), 0),
            basic::enums::Bump::Major => Version::new(version.major.saturating_add(1), 0, 0),
        }
    }
    pub(crate) fn collect_targets<'a>(
        repos: &'a [Repo],
        base: &Path,
        table: &dyn TableLike,
        targets: &mut Vec<&'a Repo>,
    ) {
        for (_, item) in table.iter() {
            let Some(entry) = item.as_table_like() else {
                continue;
            };
            let target = entry
                .get("git")
                .and_then(Item::as_str)
                .and_then(|url| repos.iter().find(|repo| repo.matches_url(url)))
                .or_else(|| entry.get("path").and_then(Item::as_str).and_then(|path| Self::resolve_repo(repos, base, path)));
            if let Some(repo) = target {
                targets.push(repo);
            }
        }
    }
    pub(crate) fn form_label(form: &basic::enums::PinForm) -> String {
        match form {
            basic::enums::PinForm::Tag(tag) => format!("tag {tag}"),
            basic::enums::PinForm::Rev(rev) => format!("rev {}", rev.chars().take(7).collect::<String>()),
            basic::enums::PinForm::Branch(branch) => format!("branch {branch}"),
            basic::enums::PinForm::Path(path) => format!("path {path}"),
            basic::enums::PinForm::Git(url) => format!("git {url}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use semver::Version;

    use super::Fleet;
    use crate::basic;

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
        let rev = basic::enums::PinForm::Rev(String::from("0123456789abcdef"));
        assert_eq!(Fleet::form_label(&rev), "rev 0123456");
        let current = Version::parse("0.0.7").unwrap();
        assert_eq!(
            Fleet::bumped(&current, basic::enums::Bump::Patch),
            Version::parse("0.0.8").unwrap()
        );
        assert_eq!(
            Fleet::bumped(&current, basic::enums::Bump::Minor),
            Version::parse("0.1.0").unwrap()
        );
        assert_eq!(
            Fleet::bumped(&current, basic::enums::Bump::Major),
            Version::parse("1.0.0").unwrap()
        );
        let stable = Version::parse("1.4.2").unwrap();
        assert_eq!(
            Fleet::bumped(&stable, basic::enums::Bump::Patch),
            Version::parse("1.4.3").unwrap()
        );
    }
}
