use crate::{
    MPath, Span,
    ast::{Block, MarstonDocument, Node, ident_table::resolve},
};
use lasso::Spur;

#[derive(Debug)]
pub struct BlockInfo {
    depth: usize,
    pub(crate) span: Span,
    pub(crate) name: Spur,
}

#[derive(Debug)]
/// Contains all the necessary information about the AST
pub struct Info {
    pub blocks: Vec<BlockInfo>,
    current_depth: usize,
}

impl Info {
    pub fn new() -> Self {
        Info { blocks: Vec::new(), current_depth: 0 }
    }

    fn enter_block(&mut self, block: Block) {
        self.blocks.push(BlockInfo {
            depth: self.current_depth,
            span: block.span,
            name: block.name.expect("shouldn't happen"),
        });
        self.current_depth += 1;
    }

    fn exit_block(&mut self) {
        self.current_depth -= 1;
    }
}

pub trait InfoWalker {
    fn collect_info(&mut self, info: &mut Info);
}

impl InfoWalker for MarstonDocument {
    fn collect_info(&mut self, p: &mut Info) {
        for block in &mut self.blocks {
            block.collect_info(p);
        }
    }
}

impl InfoWalker for Block {
    fn collect_info(&mut self, p: &mut Info) {
        p.enter_block(self.clone());

        for node in &mut self.children {
            node.collect_info(p);
        }

        p.exit_block();
    }
}

impl InfoWalker for Node {
    fn collect_info(&mut self, p: &mut Info) {
        match self {
            Node::Block(block) => {
                block.collect_info(p);
            }
            _ => {}
        }
    }
}
