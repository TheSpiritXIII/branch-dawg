use core::str;
use std::path::Path;

use git2::BranchType;
use git2::Config;
use git2::ConfigLevel;
use git2::Oid;

use crate::error;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct ReferenceInfo {
	pub name: String,
	pub oid: Oid,
}

impl ReferenceInfo {
	fn from_branch(branch: &git2::Branch) -> Result<Self, error::Error> {
		Self::from(branch.get().target().unwrap(), branch.name_bytes()?)
	}

	fn from(oid: Oid, name_bytes: &[u8]) -> Result<Self, error::Error> {
		str::from_utf8(name_bytes).map_err(error::Error::from).map(|name| {
			Self {
				name: name.to_owned(),
				oid,
			}
		})
	}
}

pub fn branches(repo: &git2::Repository) -> Result<Vec<ReferenceInfo>, error::Error> {
	repo.branches(Some(BranchType::Local))?
		.map(|branch_result| {
			branch_result
				.map_err(error::Error::from)
				.and_then(|branch| ReferenceInfo::from_branch(&branch.0))
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
