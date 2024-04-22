use core::str;
use std::collections::HashMap;
use std::fmt::Display;
use std::path::Path;

use git2::Branch;
use git2::BranchType;
use git2::Config;
use git2::ConfigLevel;
use git2::Oid;
use git2::Repository;

use crate::error;

pub enum ReferenceName {
	Branch(String),
	Tag(String),
}

impl Display for ReferenceName {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			ReferenceName::Tag(tag) => {
				f.write_str("refs/tags/")?;
				f.write_str(tag)?;
			}
			ReferenceName::Branch(branch) => {
				f.write_str("refs/heads/")?;
				f.write_str(branch)?;
			}
		}
		Ok(())
	}
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct ReferenceInfo {
	pub name: String,
	pub oid: Oid,
}

impl ReferenceInfo {
	fn from_branch(branch: &Branch) -> Result<Self, error::Error> {
		Self::from(branch_oid(branch), branch.name_bytes()?)
	}

	fn from(oid: Oid, name_bytes: &[u8]) -> Result<Self, error::Error> {
		str::from_utf8(name_bytes).map_err(error::Error::from).map(|name| {
			Self {
				name: name.to_owned(),
				oid,
			}
		})
	}

	pub fn parent(
		&self,
		repo: &Repository,
		reference_map: &HashMap<Oid, ReferenceName>,
	) -> Result<Oid, git2::Error> {
		let mut revwalk = repo.revwalk()?;
		revwalk.push(self.oid)?;
		for commit in revwalk.skip(1) {
			let oid = commit?;

			if reference_map.get(&oid).is_some() {
				return Ok(oid);
			}
		}
		Ok(Oid::zero())
	}
}

pub fn branch_oid(branch: &Branch) -> Oid {
	branch.get().target().unwrap()
}

pub fn branches(repo: &Repository) -> Result<Vec<ReferenceInfo>, error::Error> {
	repo.branches(Some(BranchType::Local))?
		.map(|branch_result| {
			branch_result
				.map_err(error::Error::from)
				.and_then(|branch| ReferenceInfo::from_branch(&branch.0))
		})
		.collect()
}

pub fn tags(repo: &Repository) -> Result<Vec<ReferenceInfo>, error::Error> {
	let mut tags = Vec::new();
	let mut err: Option<error::Error> = None;
	repo.tag_foreach(|oid, name_bytes| {
		if err.is_some() {
			return true;
		}
		match ReferenceInfo::from(oid, name_bytes) {
			Ok(info) => {
				tags.push(info);
				true
			}
			Err(e) => {
				err = Some(e);
				false
			}
		}
	})?;
	err.map_or_else(|| Ok(tags), Err)
}

pub fn branch_current(repo: &Repository) -> Result<Option<Oid>, git2::Error> {
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

pub fn commits_since(repo: &Repository, oid: Oid) -> Result<Vec<Oid>, git2::Error> {
	let mut revwalk = repo.revwalk()?;
	revwalk.push(oid)?;
	revwalk.skip(1).collect()
}

pub fn commits_to(repo: &Repository, oid_from: Oid, oid_to: Oid) -> Result<Vec<Oid>, git2::Error> {
	let mut revwalk = repo.revwalk()?;
	revwalk.push(oid_from)?;
	revwalk.skip(1).take_while(|oid| oid.is_ok() && *oid.as_ref().unwrap() != oid_to).collect()
}
