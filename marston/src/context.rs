use crate::ast::ident_table::IdentTable;
use crate::ast::parser::Parser;
use crate::config::Config;
use crate::fs::read_string;
use crate::lexer::{Token, TokenKind};
use crate::reports::ReportsBag;
use crate::{MPath, MResult};
use logos::Logos;

#[derive(Debug)]
pub struct Context {
    config: Config,
    cwd: MPath,
    ident_table: IdentTable,
    current_file: Option<MPath>,
}

impl Context {
    pub fn new(cwd: &MPath) -> MResult<Self> {
        Ok(Context {
            config: Config::find_recursively(cwd)?,
            cwd: cwd.clone(),
            ident_table: IdentTable::new(),
            current_file: None,
        })
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

    pub fn table(&self) -> &IdentTable {
        &self.ident_table
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
