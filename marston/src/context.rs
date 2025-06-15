use crate::{
    MPath, MResult,
    ast::{MarstonDocument, ident_table::IdentTable, parser::Parser},
    codegen::{Codegen, Gen},
    config::Config,
    fs::read_string,
    html::ir::ToHtmlIR,
    info::{Info, InfoWalker},
    lexer::{Token, TokenKind},
    reports::ReportsBag,
    validator::Validate,
};
use log::error;
use logos::Logos;
use std::sync::Arc;

#[derive(Debug)]
pub struct Context {
    config: Config,
    cwd: MPath,
    current_file: Option<Arc<MPath>>,
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

    pub fn file(&self) -> Arc<MPath> {
        self.current_file.clone().unwrap()
    }

    pub fn process_file(&mut self, file: &MPath) -> MResult<()> {
        self.current_file = Some(Arc::new(file.clone()));
        let content = read_string(file)?;

        ReportsBag::init(self.current_file.clone().unwrap(), Arc::<str>::from(content.clone()));
        let tokens = TokenKind::get_tokens(&content);

        let mut parser = Parser::new(self, tokens);
        parser.parse();
        let mut doc = parser.doc.clone();

        let file_name = file.strip_prefix(self.main_dir())?;
        let file = self.build_dir().join(file_name).with_extension("html");
        ReportsBag::print();

        if ReportsBag::has_errors() {
            error!("Returning errors because of errors in parsing.");
            return Ok(());
        }
        ReportsBag::clear_errors();

        let info = &mut Info::new();
        doc.collect_info(info);

        doc.validate(info);

        ReportsBag::print();

        let ir = doc.to_html_ir();
        let codegen = &mut Codegen::new();
        ir.generate(codegen);
        codegen.write_to_file(&file)?;

        Ok(())
    }
}
