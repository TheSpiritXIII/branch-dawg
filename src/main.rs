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

use std::collections::HashMap;
use std::io::stderr;
use std::io::Write;
use std::process::exit;

use clap::Args;
use clap::Parser;
use clap::Subcommand;
use git2::BranchType;
use git2::Config;
use git2::Oid;
use git2::Reference;
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

	/// Describes all tracked branches.
	Describe(CommonArgs),
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

	match cli.command {
		Commands::List(args) => list(&args),
		Commands::Describe(args) => describe(&args),
	}
}

fn list(args: &CommonArgs) {
	let mut stdout = StandardStream::stdout(ColorChoice::Always);
	let mut stderr = stderr();

	let repo = args.repository().unwrap_or_else(|e| {
		let _ = writeln!(stderr, "unable to open repository: {}", e.message());
		exit(1)
	});

	let branch_current = git_utils::branch_current(&repo).unwrap_or_else(|e| {
		let _ = writeln!(stderr, "unable to retrieve current branch: {}", e.message());
		exit(1)
	});

	let mut branches = git_utils::branches(&repo).unwrap_or_else(|e| {
		let _ = writeln!(stderr, "unable to get branches: {e}");
		exit(1)
	});

	// TODO(TheSpiritXIII): natural sorting.
	branches.sort();
	for branch in branches {
		if branch_current.is_some() && branch_current.unwrap() == branch.oid {
			let _ = write!(stdout, "* ");
			stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green))).unwrap_or_else(|e| {
				let _ = writeln!(stderr, "unable to set color: {e}");
				exit(1)
			});
		} else {
			let _ = write!(stdout, "  ");
		}
		let _ = writeln!(stdout, "{}", branch.name);

		stdout.reset().unwrap_or_else(|e| {
			let _ = writeln!(stderr, "unable to set color: {e}");
			exit(1)
		});
	}
}

fn describe(args: &CommonArgs) {
	let mut stdout = StandardStream::stdout(ColorChoice::Always);
	let mut stderr = stderr();

	let repo = args.repository().unwrap_or_else(|e| {
		let _ = writeln!(stderr, "unable to open repository: {}", e.message());
		exit(1)
	});

	let branch_current = git_utils::branch_current(&repo).unwrap_or_else(|e| {
		let _ = writeln!(stderr, "unable to retrieve current branch: {}", e.message());
		exit(1)
	});

	let config = git_utils::config_open(args.path()).unwrap_or_else(|e| {
		eprintln!("unable to read config: {}", e.message());
		exit(1)
	});

	let branch_default_name = args.branch_default_name(&config).unwrap_or_else(|e| {
		eprintln!("unable to get default branch name: {}", e.message());
		exit(1)
	});

	let branch_default =
		repo.find_branch(&branch_default_name, BranchType::Local).unwrap_or_else(|e| {
			let _ = stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)));
			let _ = writeln!(stdout, "hint: see git config init.defaultBranch");
			eprintln!("unable to get default branch name: {}", e.message());
			exit(1)
		});

	let branch_default_oid = git_utils::branch_oid(&branch_default);
	let branch_default_commits: HashMap<Oid, usize> =
		git_utils::commits_since(&repo, branch_default_oid)
			.unwrap_or_else(|e| {
				eprintln!("unable to get default branch commits: {}", e.message());
				exit(1)
			})
			.iter()
			.enumerate()
			.map(|(i, v)| (v.clone(), i))
			.collect();

	let mut branches = git_utils::branches(&repo).unwrap_or_else(|e| {
		let _ = writeln!(stderr, "unable to get branches: {e}");
		exit(1)
	});

	let tags = git_utils::tags(&repo).unwrap_or_else(|e| {
		let _ = writeln!(stderr, "unable to get tags: {e}");
		exit(1)
	});

	let reference_map = branches
		.iter()
		.map(|info| (info.oid, git_utils::ReferenceName::Branch(info.name.clone())))
		.chain(tags.iter().map(|info| (info.oid, git_utils::ReferenceName::Tag(info.name.clone()))))
		.collect();

	// TODO(TheSpiritXIII): natural sorting.
	branches.sort();
	for branch in branches {
		if branch.oid == branch_default.get().target().unwrap() {
			continue;
		}

		if branch_current.is_some() && branch_current.unwrap() == branch.oid {
			let _ = write!(stdout, "* ");
			stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green))).unwrap_or_else(|e| {
				let _ = writeln!(stderr, "unable to set color: {e}");
				exit(1)
			});
		} else {
			let _ = write!(stdout, "  ");
		}
		let _ = write!(stdout, "{} -> ", branch.name);

		let parent = branch.parent(&repo, &reference_map).unwrap();
		if parent.is_zero() {
			let _ = writeln!(stdout);
		} else {
			let main_commit = if branch_default_commits.contains_key(&parent) {
				if branch.oid == parent {
					Some(parent)
				} else {
					git_utils::commits_to(&repo, branch.oid, parent)
						.unwrap()
						.iter()
						.find(|commit| branch_default_commits.contains_key(commit))
						.cloned()
						.or(Some(parent))
				}
			} else {
				None
			};
			if let Some(commit) = main_commit {
				let _ = writeln!(stdout, "refs/heads/{branch_default_name} (detached {commit})");
			} else {
				let parent_name = reference_map.get(&parent).unwrap();
				let _ = writeln!(stdout, "{parent_name} ({parent})");
			}
		}

		stdout.reset().unwrap_or_else(|e| {
			let _ = writeln!(stderr, "unable to set color: {e}");
			exit(1)
		});
	}
}
