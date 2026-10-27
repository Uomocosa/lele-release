use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};

use lele_release::Error;
use lele_release::basic;
use lele_release::fleet::Fleet;

#[derive(Parser)]
#[command(
    name = "lele-release",
    version,
    about = "Release the lele fleet from one command"
)]
struct Args {
    #[arg(
        long,
        global = true,
        default_value = ".",
        help = "Fleet directory (the folder holding the sibling repos)"
    )]
    root: PathBuf,

    #[command(subcommand)]
    command: Cli,
}

#[derive(Subcommand)]
enum Cli {
    Status,
    Release {
        #[arg(value_parser = Cli::parse_bump)]
        level: basic::enums::Bump,
        #[arg(long, value_delimiter = ',', conflicts_with = "exclude")]
        only: Vec<String>,
        #[arg(long, value_delimiter = ',')]
        exclude: Vec<String>,
        #[arg(long)]
        dry_run: bool,
    },
    Setup,
    #[command(hide = true)]
    Pins {
        #[arg(value_parser = Cli::parse_mode)]
        mode: basic::enums::PinsMode,
        path: Option<PathBuf>,
    },
}

fn main() -> ExitCode {
    match Args::parse().run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("lele-release: {}", error.with_causes());
            ExitCode::FAILURE
        }
    }
}

#[rustfmt::skip]
impl Args {
    pub(crate) fn run(self) -> Result<(), Error> {
        let root = Self::fleet_root(&self.root);
        match self.command {
            Cli::Status => Fleet::try_from(root.as_path())?.status(),
            Cli::Release { level, only, exclude, dry_run } => {
                Fleet::try_from(root.as_path())?.release(level, &Cli::selection(only, exclude), dry_run)
            }
            Cli::Setup => Fleet::try_from(root.as_path())?.setup(),
            Cli::Pins { mode, path } => Fleet::filter_stdin_or_pass_through(mode, path.as_deref(), &root),
        }
    }
    pub(crate) fn fleet_root(repo_or_fleet_dir: &Path) -> PathBuf {
        let absolute = repo_or_fleet_dir.canonicalize().unwrap_or_else(|_| repo_or_fleet_dir.to_path_buf());
        if absolute.join(".git").exists() {
            return absolute.parent().unwrap_or(absolute.as_path()).to_path_buf();
        }
        absolute
    }
}

#[rustfmt::skip]
impl Cli {
    pub(crate) fn selection(only: Vec<String>, exclude: Vec<String>) -> basic::enums::Selection {
        if !only.is_empty() {
            return basic::enums::Selection::Only(only);
        }
        if !exclude.is_empty() {
            return basic::enums::Selection::Exclude(exclude);
        }
        basic::enums::Selection::All
    }
    pub(crate) fn parse_bump(value: &str) -> Result<basic::enums::Bump, String> {
        match value {
            "patch" => Ok(basic::enums::Bump::Patch),
            "minor" => Ok(basic::enums::Bump::Minor),
            "major" => Ok(basic::enums::Bump::Major),
            other => Err(format!("expected patch|minor|major, got `{other}`")),
        }
    }
    pub(crate) fn parse_mode(value: &str) -> Result<basic::enums::PinsMode, String> {
        match value {
            "clean" => Ok(basic::enums::PinsMode::Clean),
            "smudge" => Ok(basic::enums::PinsMode::Smudge),
            other => Err(format!("expected clean|smudge, got `{other}`")),
        }
    }
}
