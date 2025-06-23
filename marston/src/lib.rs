extern crate core;

#[feature(let_chains)]
use camino::Utf8PathBuf;
use std::ops::Range;

mod ast;
mod codegen;
pub mod config;
pub mod context;
pub mod fs;
pub mod html;
mod info;
pub mod lexer;
mod reports;
mod span;
mod validator;

pub type MResult<T> = anyhow::Result<T>;

pub type MPath = Utf8PathBuf;
pub type Span = Range<usize>;
