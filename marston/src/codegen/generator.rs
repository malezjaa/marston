use crate::{
    ast::{Block, MarstonDocument, Node, ident_table::resolve},
    codegen::{Codegen, Gen},
};
use v_htmlescape::escape;

impl Gen for MarstonDocument {
    fn generate(&self, p: &mut Codegen) {
        p.top_level = true;
        p.writeln("<!DOCTYPE html>");
        p.top_level = false;

        p.writeln("<html>");
        p.indent();

        for block in &self.blocks {
            block.generate(p);
        }

        p.dedent();
        p.writeln("</html>");
    }
}

impl Gen for Block {
    fn generate(&self, p: &mut Codegen) {
        if let Some(ref name) = self.name {
            let tag = resolve(name.key);

            p.writeln(&format!("<{}>", tag));
            p.indent();

            for node in &self.children {
                node.generate(p);
            }

            p.dedent();

            p.writeln(&format!("</{}>", tag));
        }
    }
}

impl Gen for Node {
    fn generate(&self, p: &mut Codegen) {
        p.top_level = false;

        match self {
            Node::Block(block) => block.generate(p),
            Node::Text(text) => p.writeln(&escape(text).to_string()),
        }
    }
}
