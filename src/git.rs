use std::{
	collections::{HashMap, HashSet},
	fmt::Display,
};

use git2::{Branch, BranchType, Oid, Repository};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum BranchError {
	#[error("git error: {0}")]
	Git(#[from] git2::Error),
	#[error("unable to convert branch with hash {0}")]
	ConversionError(Oid),
}

// TODO: must know about main branch.
fn build(repo: &Repository, branch_default_name: &str) -> Result<(), BranchError> {
	let branch_default = repo.find_branch(branch_default_name, BranchType::Local)?;
	let remote_map = reference_map(repo, &branch_default)?;

	let mut mappings = HashMap::<String, String>::new();
	for branch in repo.branches(Some(BranchType::Local))? {
		let b = branch?.0;
		if b.get() == branch_default.get() {
			continue;
		}
		// let b = &branch.as_ref().unwrap().0;
		let name = b.name()?.unwrap().to_string();
		mappings.insert(name, find_parent(repo, &b, &remote_map)?);
	}
	for (k, v) in mappings {
		println!("Branch: {}, parent \"{}\"", k, v)
	}
	Ok(())
}

enum Reference {
	Tag(String),
	Branch(String),
	Default,
}

impl Display for Reference {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Reference::Tag(tag) => {
				f.write_str("refs/tags/")?;
				f.write_str(tag)?;
			},
			Reference::Branch(branch) => {
				f.write_str("refs/heads/")?;
				f.write_str(branch)?;
			},
			Reference::Default => {
				f.write_str("refs/default")?;
			},
		}
		Ok(())
	}
}

fn reference_map(repo: &Repository, branch_default: &Branch) -> Result<HashMap<Oid, Reference>, BranchError> {
	let mut revwalk = repo.revwalk()?;
	let oid = branch_default.get().target().unwrap();
	revwalk.push(oid)?;

	repo.references()?
		.filter_map(|ref_result| match ref_result {
			Ok(r) => {
				if r.is_branch() {
					let oid = r.resolve().unwrap().target().unwrap();
					let name = r.shorthand().unwrap().to_owned();
					Some(Ok((oid, Reference::Branch(name))))
				} else if r.is_tag() {
					let oid = r.resolve().unwrap().target().unwrap();
					let name = r.shorthand().unwrap().to_owned();
					Some(Ok((oid, Reference::Tag(name))))
				} else {
					None
				}
			}
			Err(e) => Some(Err(e.into())),
		})
		.chain(revwalk.map(|c| {
			Ok((c?, Reference::Default))
		}))
		.collect()
}

fn branch_commit_list(repo: &Repository, branch: &Branch) -> Result<HashSet<Oid>, git2::Error> {
	let mut revwalk = repo.revwalk()?;
	let oid = branch.get().target().unwrap();
	revwalk.push(oid)?;
	revwalk.collect()
}

fn branch_commit_oid(branch: &Branch) -> Result<Oid, git2::Error> {
	// unwrap is safe here because we resolve to a direct reference.
	Ok(branch.get().resolve()?.target().unwrap())
}

// TODO: sometimes parent is a SHA (from main branch).
fn find_parent(
	repo: &Repository,
	branch: &Branch,
	reference_map: &HashMap<Oid, Reference>,
) -> Result<String, git2::Error> {
	// let name = branch.name()?.unwrap().to_string();
	let branch_oid = branch.get().resolve()?.target().unwrap();
	let mut revwalk = repo.revwalk()?;
	let oid = branch.get().target().unwrap();
	revwalk.push(oid)?;
	for commit in revwalk {
		let oid2 = commit?;
		if oid2 == branch_oid {
			continue;
		}

		if let Some(m) = reference_map.get(&oid2) {
			return Ok(format!("{} ({})", m, oid2));
		}
	}
	// Not part of main branch. Ignore?
	return Ok(String::new());
}
