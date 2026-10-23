use std::io::Read;
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
        #[arg(value_parser = parse_bump)]
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
        #[arg(value_parser = parse_mode)]
        mode: basic::enums::PinsMode,
        path: Option<PathBuf>,
    },
}

fn main() -> ExitCode {
    match run(Args::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("lele-release: {error}");
            ExitCode::FAILURE
        }
    }
}

// needed helper: dispatch
fn run(args: Args) -> Result<(), Error> {
    let root = fleet_root(&args.root);
    match args.command {
        Cli::Status => Fleet::try_from(root.as_path())?.status(),
        Cli::Release {
            level,
            only,
            exclude,
            dry_run,
        } => Fleet::try_from(root.as_path())?.release(level, &selection(only, exclude), dry_run),
        Cli::Setup => Fleet::try_from(root.as_path())?.setup(),
        Cli::Pins { mode, path } => pins(mode, path.as_deref(), &root),
    }
}

// needed helper: a repo directory means "fleet root is its parent"
fn fleet_root(given: &Path) -> PathBuf {
    let absolute = given.canonicalize().unwrap_or_else(|_| given.to_path_buf());
    if absolute.join(".git").exists() {
        return absolute
            .parent()
            .unwrap_or(absolute.as_path())
            .to_path_buf();
    }
    absolute
}

// needed helper: --only and --exclude are mutually exclusive at the CLI level
fn selection(only: Vec<String>, exclude: Vec<String>) -> basic::enums::Selection {
    if !only.is_empty() {
        return basic::enums::Selection::Only(only);
    }
    if !exclude.is_empty() {
        return basic::enums::Selection::Exclude(exclude);
    }
    basic::enums::Selection::All
}

// needed helper: parse the bump level without clap's ValueEnum (it stays a plain enum)
fn parse_bump(value: &str) -> Result<basic::enums::Bump, String> {
    match value {
        "patch" => Ok(basic::enums::Bump::Patch),
        "minor" => Ok(basic::enums::Bump::Minor),
        "major" => Ok(basic::enums::Bump::Major),
        other => Err(format!("expected patch|minor|major, got `{other}`")),
    }
}

// needed helper: parse the hidden pins mode
fn parse_mode(value: &str) -> Result<basic::enums::PinsMode, String> {
    match value {
        "clean" => Ok(basic::enums::PinsMode::Clean),
        "smudge" => Ok(basic::enums::PinsMode::Smudge),
        other => Err(format!("expected clean|smudge, got `{other}`")),
    }
}

// needed helper: the git filter entry point; never fails a git operation
fn pins(mode: basic::enums::PinsMode, path: Option<&Path>, root: &Path) -> Result<(), Error> {
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .map_err(|error| Error::Io {
            message: format!("stdin: {error}"),
        })?;
    let Some(path) = path else {
        print!("{input}");
        return Ok(());
    };
    let Ok(cwd) = std::env::current_dir() else {
        print!("{input}");
        return Ok(());
    };
    let manifest = cwd.join(path);
    let Ok(fleet) = Fleet::try_from(root) else {
        print!("{input}");
        return Ok(());
    };
    let is_lock = path.file_name().and_then(|name| name.to_str()) == Some("Cargo.lock");
    let output = match (mode, is_lock) {
        (basic::enums::PinsMode::Clean, true) => fleet.clean_lock(&manifest, &input),
        (basic::enums::PinsMode::Smudge, true) => fleet.smudge_lock(&manifest, &input),
        (basic::enums::PinsMode::Clean, false) => fleet.clean_manifest(&manifest, &input),
        (basic::enums::PinsMode::Smudge, false) => fleet.smudge_manifest(&manifest, &input),
    };
    match output {
        Ok(text) => print!("{text}"),
        Err(_) => print!("{input}"),
    }
    Ok(())
}
