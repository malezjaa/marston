use crate::MResult;
use crate::fs::walk_for_file;
use anyhow::anyhow;
use fs_err::read_to_string;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub project: ProjectConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
}

impl Config {
    const FILE_NAME: &'static str = "marston.toml";

    pub fn find_recursively(cwd: PathBuf) -> MResult<Self> {
        let file = walk_for_file(cwd.clone(), Self::FILE_NAME).ok_or_else(|| {
            anyhow!("No config file found in {} or any of its parents", cwd.display())
        })?;
        let content = read_to_string(file)?;
        Ok(Config::fill_defaults(toml::from_str::<Config>(&content)?))
    }

    fn fill_defaults(config: Config) -> Self {
        config
    }
}
