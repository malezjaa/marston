use crate::{
    ast::{
        Block, MarstonDocument, Node, ValueKind,
        ident_table::{get_or_intern, resolve},
    },
    codegen::{Codegen, Gen},
    html::ir::{IrDoc, IrNode},
};
use v_htmlescape::escape;

impl Gen for IrDoc {
    fn generate(&self, p: &mut Codegen) {
        p.top_level = true;
        p.writeln("<!DOCTYPE html>");
        p.top_level = false;

        for block in &self.root {
            block.generate(p);
        }
    }
}

impl Gen for IrNode {
    fn generate(&self, p: &mut Codegen) {
        match self {
            IrNode::Element(element) => {
                let tag = resolve(element.tag);
                let attrs = element
                    .attributes
                    .iter()
                    .map(|attr| {
                        if let ValueKind::Boolean(bool) = attr.value {
                            if bool { format!("{}", resolve(attr.key)) } else { "".to_string() }
                        } else {
                            format!("{}=\"{}\"", resolve(attr.key), &attr.value.to_string())
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ");

                let space = if attrs.is_empty() { "" } else { " " };

                if element.children.is_empty() {
                    p.writeln(&format!("<{tag}{space}{attrs}/>"));
                } else {
                    p.writeln(&format!("<{tag}{space}{attrs}>"));
                    p.indent();

                    for node in &element.children {
                        node.generate(p);
                    }

                    p.dedent();

                    p.writeln(&format!("</{tag}>"));
                }
            }
            IrNode::Text(text) => p.writeln(&escape(text).to_string()),
        }
    }
}
