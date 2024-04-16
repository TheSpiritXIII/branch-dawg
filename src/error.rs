#[derive(thiserror::Error, Debug)]
pub enum Error {
	#[error("Git: {}", .0.message())]
	Git(#[from] git2::Error),

	#[error("Unable to convert UTF-8 at index {}", .0.valid_up_to())]
	Utf8Error(#[from] std::str::Utf8Error),
}
