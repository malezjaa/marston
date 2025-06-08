use crate::config::Config;
use crate::fs::read_string;
use crate::lexer::Token;
use crate::reports::ReportsBag;
use crate::{MPath, MResult, error_report};
use ariadne::Label;
use ariadne::ReportKind;
use ariadne::{Color, Fmt, Report};
use logos::Logos;
use std::borrow::Cow;

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

        let mut bag = ReportsBag::new(file, content.as_str());
        let tokens = Token::get_tokens(&content);

        bag.print();
        Ok(())
    }
}
