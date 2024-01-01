use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
	#[error("io error")]
	IO(String),
}
