use crate::{
    ast::MarstonDocument,
    codegen::{Codegen, Gen},
};

impl Gen for MarstonDocument {
    fn generate(&self, p: &mut Codegen) {
        p.writeln("<!DOCTYPE html>");
    }
}
