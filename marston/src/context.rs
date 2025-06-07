use crate::{MPath, MResult};
use crate::config::Config;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Context {
    config: Config,
}

impl Context {
    pub fn new(cwd: &MPath) -> MResult<Self> {
        Ok(Context { config: Config::find_recursively(cwd)? })
    }

    pub fn name(&self) -> String {
        self.config.project.name.clone()
    }
}
