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
}

#[derive(Args)]
struct ListArgs {
	/// The path to the Git repository, or the current path if none is provided.
	path: Option<String>,
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
			let branches = match repo.branches(Some(BranchType::Local)) {
				Ok(b) => b,
				Err(e) => {
					println!("unable to list branches: {}", e.message());
					return;
				}
			};
			for branch in branches {
				let b = match branch {
					Ok(b) => b.0,
					Err(e) => {
						println!("unable to list branches: {}", e.message());
						return;
					}
				};
				let name = match b.name() {
					Ok(n) => {
						match n {
							Some(n) => n,
							None => {
								println!("branch has invalid name");
								return;
							}
						}
					}
					Err(e) => {
						println!("unable to retrieve branch name: {}", e.message());
						return;
					}
				};
				println!("{name}");
			}
		}
	}
}
