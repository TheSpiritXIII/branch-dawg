//! Power tool for managing your git branches.
//!
//! ![Yo Dawg Meme](yodawg.jpg)
//!
//! # Usage
//!
//! > ⚠️ Warning: This tool is a WIP. The API may change at any time and may break your branches.
//!
//! To list all branches in a git repository, run:
//!
//! ```
//! branch-dawg list
//! ```
//!
//! This outputs all local branches.

use std::io::Write;

use clap::Args;
use clap::Parser;
use clap::Subcommand;
use git2::BranchType;
use git2::Repository;
use termcolor::Color;
use termcolor::ColorChoice;
use termcolor::ColorSpec;
use termcolor::StandardStream;
use termcolor::WriteColor;
use thiserror::Error;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
	#[command(subcommand)]
	command: Commands,
}

#[derive(Subcommand)]
enum Commands {
	/// Lists all tracked branches.
	List(CommonArgs),
}

#[derive(Args)]
struct CommonArgs {
	/// The path to the Git repository, or the current path if none is provided.
	path: Option<String>,

	/// The default branch, or `main` if none is provided.
	default_branch: Option<String>,
}

fn main() {
	let cli = Cli::parse();

	let mut stdout = StandardStream::stdout(ColorChoice::Always);

	match cli.command {
		Commands::List(args) => {
			let path = args.path.unwrap_or(String::new());
			let repo = match Repository::open(path) {
				Ok(r) => r,
				Err(e) => {
					println!("unable to open repository: {}", e.message());
					return;
				}
			};

			let branch_default_name = args.default_branch.unwrap_or("main".to_owned());

			match list_branches(&repo) {
				Ok(mut branches) => {
					// TODO(TheSpiritXIII): natural sorting.
					branches.sort();
					for name in branches {
						if name == branch_default_name {
							write!(stdout, "* ").unwrap();
							stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green))).unwrap();
						} else {
							write!(stdout, "  ").unwrap();
						}
						writeln!(stdout, "{name}").unwrap();
						stdout.reset().unwrap();
					}
				}
				Err(e) => {
					eprintln!("{}", e)
				}
			}
		}
	}
}

#[derive(Error, Debug)]
pub enum CliError {
	#[error("Git: {}", .0.message())]
	Git(#[from] git2::Error),

	#[error("Unable to convert UTF-8 at index {}", .0.valid_up_to())]
	Utf8Error(#[from] std::str::Utf8Error),
}

fn list_branches(repo: &git2::Repository) -> Result<Vec<String>, CliError> {
	repo.branches(Some(BranchType::Local))?
		.map(|branch_result| {
			branch_result.map_err(CliError::from).and_then(|branch| {
				return branch.0.name_bytes().map_err(CliError::from).and_then(|name| {
					std::str::from_utf8(name).map_err(CliError::from).map(str::to_owned)
				});
			})
		})
		.collect()
}
