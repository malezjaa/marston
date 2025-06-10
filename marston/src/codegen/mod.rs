mod generator;

use crate::{MPath, MResult, ast::MarstonDocument};
use std::{
    fs::{self, File, create_dir_all},
    io::Write,
    path::Path,
};
use std::io::BufWriter;

pub struct Codegen {
    content: BufWriter<Vec<u8>>,
    indent_level: usize,
    indent_size: usize,
}

impl Codegen {
    pub fn new() -> Self {
        Self {
            content: BufWriter::new(vec![]),
            indent_level: 0,
            indent_size: 4,
        }
    }
    pub fn write_to_file(&self, path: &MPath) -> MResult<()> {
        if let Some(parent) = path.parent() {
            create_dir_all(parent)?;
        }

        let mut file = File::create(path)?;
        file.write_all(self.content.buffer())?;

        Ok(())
    }

    pub fn write(&mut self, text: &str) -> &mut Self {
        write!(self.content, "{}", text).unwrap();
        self
    }

    pub fn writeln(&mut self, text: &str) -> &mut Self {
        let indent = " ".repeat(self.indent_level * self.indent_size);

        writeln!(self.content, "{}{}", indent, text).unwrap();
        self
    }

    pub fn newline(&mut self) -> &mut Self {
        self.writeln("");
        self
    }

    pub fn indent(&mut self) -> &mut Self {
        self.indent_level += 1;
        self
    }

    pub fn dedent(&mut self) -> &mut Self {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
        self
    }
}

pub trait Gen {
    fn generate(&self, p: &mut Codegen);
}
