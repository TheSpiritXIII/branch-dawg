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

#![warn(clippy::pedantic)]

use std::io::Write;

use clap::Args;
use clap::Parser;
use clap::Subcommand;
use git2::Config;
use git2::Repository;
use termcolor::Color;
use termcolor::ColorChoice;
use termcolor::ColorSpec;
use termcolor::StandardStream;
use termcolor::WriteColor;

mod error;
mod git_utils;

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

impl CommonArgs {
	fn path(&self) -> &str {
		self.path.as_deref().unwrap_or("./")
	}

	fn repository(&self) -> Result<Repository, git2::Error> {
		Repository::open(self.path())
	}

	fn branch_default_name(&self, config: &Config) -> Result<String, git2::Error> {
		if let Some(name) = &self.default_branch {
			Ok(name.clone())
		} else {
			let config_default = config.get_str("init.defaultBranch")?;
			if config_default.is_empty() {
				Ok("main".to_owned())
			} else {
				Ok(config_default.to_owned())
			}
		}
	}
}

fn main() {
	let cli = Cli::parse();

	let mut stdout = StandardStream::stdout(ColorChoice::Always);

	match cli.command {
		Commands::List(args) => {
			let repo = match args.repository() {
				Ok(repo) => repo,
				Err(e) => {
					eprintln!("unable to open repository: {}", e.message());
					return;
				}
			};

			let config = match git_utils::config_open(args.path()) {
				Ok(config) => config,
				Err(e) => {
					eprintln!("unable to open config: {}", e.message());
					return;
				}
			};

			let branch_default_name = match args.branch_default_name(&config) {
				Ok(config) => config,
				Err(e) => {
					eprintln!("unable to retrieve default branch: {}", e.message());
					return;
				}
			};

			match git_utils::list_branches(&repo) {
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
					eprintln!("{e}");
				}
			}
		}
	}
}
