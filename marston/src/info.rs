#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(pub usize);

use crate::{
    MPath, Span,
    ast::{Block, Interned, MarstonDocument, Node},
};
use lasso::Spur;
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct BlockInfo {
    pub id: usize,
    pub name: Interned,
    pub span: Span,
    pub depth: usize,
    pub parent: Option<usize>,
    pub children: Vec<usize>,
}

#[derive(Debug)]
pub struct Info {
    blocks: Vec<BlockInfo>,
    name_index: HashMap<Spur, Vec<usize>>,
    id_counter: usize,
    current_parent: Option<usize>,
    current_depth: usize,
}

impl Info {
    pub fn new() -> Self {
        Self {
            blocks: vec![],
            name_index: HashMap::new(),
            id_counter: 0,
            current_parent: None,
            current_depth: 0,
        }
    }

    pub fn has_block(&self, name: Spur) -> bool {
        self.name_index.contains_key(&name)
    }

    pub fn all_blocks_named(&self, name: Spur) -> Vec<&BlockInfo> {
        self.name_index
            .get(&name)
            .map(|ids| ids.iter().map(|&i| &self.blocks[i]).collect())
            .unwrap_or_default()
    }

    pub fn find_blocks_by_name_and_parent(&self, name: Spur, parent_name: Spur) -> Vec<&BlockInfo> {
        self.name_index.get(&name).map_or(vec![], |ids| {
            ids.iter()
                .filter_map(|&id| {
                    let block = &self.blocks[id];
                    block.parent.and_then(|pid| {
                        let parent = &self.blocks[pid];
                        if parent.name.key == parent_name { Some(block) } else { None }
                    })
                })
                .collect()
        })
    }

    fn next_id(&mut self) -> usize {
        let id = self.id_counter;
        self.id_counter += 1;
        id
    }

    fn enter_block(&mut self, block: Block) -> usize {
        let id = self.next_id();

        let info = BlockInfo {
            id,
            name: block.name.expect("block should have name"),
            span: block.span,
            depth: self.current_depth,
            parent: self.current_parent,
            children: vec![],
        };

        if let Some(p) = self.current_parent {
            self.blocks[p].children.push(id);
        }

        self.name_index.entry(info.name.key).or_default().push(id);
        self.blocks.push(info);

        self.current_depth += 1;
        let old_parent = self.current_parent;
        self.current_parent = Some(id);
        old_parent.unwrap_or(id)
    }

    fn exit_block(&mut self, old_parent: usize) {
        self.current_depth -= 1;
        self.current_parent = Some(old_parent);
    }

    pub fn blocks(&self) -> &[BlockInfo] {
        &self.blocks
    }
}

pub trait InfoWalker {
    fn collect_info(&mut self, info: &mut Info);
}

impl InfoWalker for MarstonDocument {
    fn collect_info(&mut self, info: &mut Info) {
        for block in &mut self.blocks {
            block.collect_info(info);
        }
    }
}

impl InfoWalker for Block {
    fn collect_info(&mut self, info: &mut Info) {
        let old_parent = info.enter_block(self.clone());

        for child in &mut self.children {
            child.collect_info(info);
        }

        info.exit_block(old_parent);
    }
}

impl InfoWalker for Node {
    fn collect_info(&mut self, info: &mut Info) {
        if let Node::Block(block) = self {
            block.collect_info(info);
        }
    }
}
