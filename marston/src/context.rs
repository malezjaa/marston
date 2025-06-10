use crate::{
    MPath, MResult,
    ast::{ident_table::IdentTable, parser::Parser},
    config::Config,
    fs::read_string,
    lexer::{Token, TokenKind},
    reports::ReportsBag,
};
use logos::Logos;

#[derive(Debug)]
pub struct Context {
    config: Config,
    cwd: MPath,
    current_file: Option<MPath>,
}

impl Context {
    pub fn new(cwd: &MPath) -> MResult<Self> {
        Ok(Context { config: Config::find_recursively(cwd)?, cwd: cwd.clone(), current_file: None })
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

    pub fn file(&self) -> &MPath {
        self.current_file.as_ref().expect("current file is not set")
    }

    pub fn process_file(&mut self, file: &MPath) -> MResult<()> {
        self.current_file = Some(file.clone());
        let content = read_string(file)?;

        let mut bag = ReportsBag::new(file, content.as_str());
        let tokens = TokenKind::get_tokens(&content);

        let mut parser = Parser::new(self, tokens);
        parser.parse();
        bag.extend(parser.bag);
        let doc = parser.doc.clone();

        bag.print();
        Ok(())
    }
}
