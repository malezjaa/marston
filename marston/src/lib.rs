#[feature(let_chains)]
use camino::Utf8PathBuf;
use std::ops::Range;

mod ast;
pub mod config;
pub mod context;
pub mod fs;
pub mod lexer;
mod reports;
mod span;

pub type MResult<T> = anyhow::Result<T>;

pub type MPath = Utf8PathBuf;
pub type Span = Range<usize>;
