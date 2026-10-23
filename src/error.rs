use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("git failed in {repo}: {command}\n{stderr}")]
    Git {
        repo: String,
        command: String,
        stderr: String,
    },

    #[error("{context}")]
    Io {
        context: String,
        #[source]
        source: std::io::Error,
    },

    #[error("cannot parse {}", path.display())]
    Toml {
        path: PathBuf,
        #[source]
        source: toml_edit::TomlError,
    },

    #[error("{context}")]
    CommitCount {
        context: String,
        #[source]
        source: std::num::ParseIntError,
    },

    #[error("{context}")]
    Semver {
        context: String,
        #[source]
        source: semver::Error,
    },

    #[error("{0}")]
    Refused(String),
}
