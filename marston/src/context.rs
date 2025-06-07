use crate::config::Config;
use crate::fs::read_string;
use crate::lexer::Token;
use crate::{MPath, MResult};
use logos::Logos;

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

    pub fn process_file(&self, file: &MPath) -> MResult<()> {
        let content = read_string(file)?;
        let tokens = Token::get_tokens(content.as_str());

        Ok(())
    }
}
