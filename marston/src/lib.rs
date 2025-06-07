use camino::Utf8PathBuf;

pub mod config;
pub mod context;
pub mod fs;

pub type MResult<T> = anyhow::Result<T>;

pub type MPath = Utf8PathBuf;
