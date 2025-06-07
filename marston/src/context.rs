use std::path::{Path, PathBuf};
use crate::config::Config;
use crate::MResult;

#[derive(Debug)]
pub struct Context {
    config: Config
}

impl Context {
    pub fn new(cwd: PathBuf) -> MResult<Self> {
        Ok(Context {
            config: Config::find_recursively(cwd)?
        })
    }
}