use core::str;
use std::path::Path;

use git2::BranchType;
use git2::Config;
use git2::ConfigLevel;

use crate::error;

pub fn list_branches(repo: &git2::Repository) -> Result<Vec<String>, error::Error> {
	repo.branches(Some(BranchType::Local))?
		.map(|branch_result| {
			branch_result.map_err(error::Error::from).and_then(|branch| {
				return branch.0.name_bytes().map_err(error::Error::from).and_then(|name| {
					str::from_utf8(name).map_err(error::Error::from).map(str::to_owned)
				});
			})
		})
		.collect()
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
