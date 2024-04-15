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

use clap::Args;
use clap::Parser;
use clap::Subcommand;
use git2::BranchType;
use git2::Repository;

mod error;
mod git;

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
	List(ListArgs),

	/// Rebases all tracked branches.
	Rebase(RebaseArgs),
}

#[derive(Args)]
struct ListArgs {
	/// The path to the Git repository, or the current path if none is provided.
	path: Option<String>,
}

#[derive(Args)]
struct RebaseArgs {
	/// The path to the Git repository, or the current path if none is provided.
	path: Option<String>,

	/// The default branch, or `main` if none is provided.
	default_branch: Option<String>,
}

fn main() {
	let cli = Cli::parse();

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
			match list_branches(&repo) {
				Ok(branches) => {
					for name in branches {
						println!("{name}");
					}
				}
				Err(e) => {
					eprintln!("{}", e)
				}
			}
		},
		Commands::Rebase(args) => {
			let path = args.path.unwrap_or(String::new());
			let repo = match Repository::open(path) {
				Ok(r) => r,
				Err(e) => {
					println!("unable to open repository: {}", e.message());
					return;
				}
			};

			let branch_default_name = args.default_branch.unwrap_or("main".to_owned());
			let branch_default = match repo.find_branch(&branch_default_name, BranchType::Local) {
				Ok(b) => b,
				Err(e) => {
					println!("unable to retrieve default branch {}: {}", branch_default_name, e.message());
					return;
				}
			};

			println!("Note: This experimental command only rebases branches with no children!");

		},
	}
}

fn list_branches(repo: &git2::Repository) -> Result<Vec<String>, error::Error> {
	repo.branches(Some(BranchType::Local))?
		.map(|branch_result| {
			branch_result.map_err(error::Error::from).and_then(|branch| {
				return branch.0.name_bytes().map_err(error::Error::from).and_then(|name| {
					std::str::from_utf8(name).map_err(error::Error::from).map(str::to_owned)
				});
			})
		})
		.collect()
}
