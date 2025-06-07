use crate::{MPath, MResult};
use crate::config::Config;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Context {
    config: Config,
    cwd: MPath,
}

impl Context {
    pub fn new(cwd: &MPath) -> MResult<Self> {
        Ok(Context { config: Config::find_recursively(cwd)?, cwd: cwd.clone() })
    }

    pub fn name(&self) -> String {
        self.config.project.name.clone()
    }
    
    pub fn build_dir(&self) -> &MPath {
        &self.config.build.output_dir
    }
    
    pub fn main_dir(&self) -> &MPath {
        &self.config.build.main_dir
    }
}
