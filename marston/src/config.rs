use crate::{MPath, MResult, fs::walk_for_file};
use anyhow::anyhow;
use fs_err::read_to_string;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedConfig {
    pub project: ProjectConfig,
    pub build: Option<BuildConfig>,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub project: ProjectConfig,
    pub build: BuildConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    pub output_dir: MPath,
    pub main_dir: MPath,
}

impl Config {
    const FILE_NAME: &'static str = "marston.toml";

    pub fn find_recursively(cwd: &MPath) -> MResult<Self> {
        let file = walk_for_file(cwd.into(), Self::FILE_NAME).ok_or_else(|| {
            anyhow!("No config file found in {} or any of its parents", cwd.to_string())
        })?;
        let content = read_to_string(file)?;
        Ok(Config::fill_defaults(toml::from_str::<ParsedConfig>(&content)?, cwd))
    }

    fn fill_defaults(config: ParsedConfig, cwd: &MPath) -> Self {
        let build = config
            .build
            .unwrap_or(BuildConfig { output_dir: cwd.join("dist"), main_dir: cwd.join("src") });

        Self { project: config.project, build }
    }
}
