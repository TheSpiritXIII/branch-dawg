use core::str;
use std::path::Path;

use git2::BranchType;
use git2::Config;
use git2::ConfigLevel;
use git2::Oid;

use crate::error;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Branch {
	pub name: String,
	pub oid: Oid,
}

pub fn branches(repo: &git2::Repository) -> Result<Vec<Branch>, error::Error> {
	repo.branches(Some(BranchType::Local))?
		.map(|branch_result| {
			branch_result.map_err(error::Error::from).and_then(|branch| {
				return branch.0.name_bytes().map_err(error::Error::from).and_then(|name| {
					str::from_utf8(name).map_err(error::Error::from).map(|name| {
						Branch {
							name: name.to_owned(),
							oid: branch.0.get().target().unwrap(),
						}
					})
				});
			})
		})
		.collect()
}

pub fn branch_current(repo: &git2::Repository) -> Result<Option<Oid>, git2::Error> {
	let head = repo.head()?;
	head.is_branch()
		.then(|| head.resolve())
		.transpose()
		.map(|ref_resolved| ref_resolved.map(|x| x.target().unwrap()))
}

pub fn config_open(path: impl AsRef<Path>) -> Result<Config, git2::Error> {
	let mut config = Config::new()?;
	let config_path = path.as_ref().join(".git/config");
	if let Ok(metadata) = config_path.metadata() {
		if metadata.is_file() {
			config.add_file(&config_path, ConfigLevel::Local, true)?;
		}
	}
	if let Ok(path_found) = Config::find_global() {
		config.add_file(&path_found, ConfigLevel::Global, true)?;
	}
	if let Ok(path_found) = Config::find_system() {
		config.add_file(&path_found, ConfigLevel::System, true)?;
	}
	if let Ok(path_found) = Config::find_xdg() {
		config.add_file(&path_found, ConfigLevel::XDG, true)?;
	}
	config.snapshot()
}
