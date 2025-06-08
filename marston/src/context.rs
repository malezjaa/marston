use crate::ast::ident_table::IdentTable;
use crate::config::Config;
use crate::fs::read_string;
use crate::lexer::Token;
use crate::reports::ReportsBag;
use crate::{MPath, MResult};
use logos::Logos;

#[derive(Debug)]
pub struct Context<'a> {
    config: Config,
    cwd: MPath,
    bag: Option<ReportsBag<'a>>,
    ident_table: IdentTable,
}

impl<'a> Context<'a> {
    pub fn new(cwd: &MPath) -> MResult<Self> {
        Ok(Context {
            config: Config::find_recursively(cwd)?,
            cwd: cwd.clone(),
            bag: None,
            ident_table: IdentTable::new(),
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

    pub fn current_bag(&self) -> &ReportsBag {
        self.bag.as_ref().expect("reports bag is not set")
    }

    pub fn table(&self) -> &IdentTable {
        &self.ident_table
    }

    pub fn process_file(&self, file: &MPath) -> MResult<()> {
        let content = read_string(file)?;

        let mut bag = ReportsBag::new(file, content.as_str());
        let tokens = Token::get_tokens(&content);

        bag.print();
        Ok(())
    }
}
