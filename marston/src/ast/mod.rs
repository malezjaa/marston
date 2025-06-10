use crate::Span;
use lasso::{Key, Spur};
use rustc_hash::FxHashMap;

pub mod ident_table;
pub mod parser;

#[derive(Debug, Clone)]
pub struct MarstonDocument {
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone)]
pub enum Node {
    Block(Block),
    Text(String),
}

#[derive(Debug, Clone)]
pub struct Block {
    pub name: Option<Spur>,
    pub attributes: FxHashMap<Spur, Value>,
    pub children: Vec<Node>,
    pub span: Option<Span>,
}

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Number(f64),
    Boolean(bool),
    Array(Vec<Value>),
}

impl MarstonDocument {
    pub fn new() -> Self {
        Self { blocks: Vec::new() }
    }

    pub fn add_block(&mut self, block: Block) {
        self.blocks.push(block);
    }

    pub fn find_block_by_name(&self, name: Spur) -> Option<&Block> {
        self.blocks.iter().find(|e| e.name == Some(name))
    }

    pub fn find_blocks_by_name(&self, name: Spur) -> Vec<&Block> {
        self.blocks.iter().filter(|e| e.name == Some(name)).collect()
    }
}

impl Block {
    pub fn new(name: Option<Spur>) -> Self {
        Self { name, attributes: FxHashMap::default(), children: Vec::new(), span: None }
    }

    pub fn with_attributes(name: Option<Spur>, attributes: FxHashMap<Spur, Value>) -> Self {
        Self { name, attributes, children: Vec::new(), span: None }
    }

    pub fn add_attribute(&mut self, key: Spur, value: Value) {
        self.attributes.insert(key, value);
    }

    pub fn get_attribute(&self, key: Spur) -> Option<&Value> {
        self.attributes.get(&key)
    }

    pub fn add_child(&mut self, child: Node) {
        self.children.push(child);
    }

    pub fn add_text(&mut self, text: String) {
        self.children.push(Node::Text(text));
    }

    pub fn add_block(&mut self, block: Block) {
        self.children.push(Node::Block(block));
    }

    pub fn find_child_block(&self, name: Spur) -> Option<&Block> {
        self.children.iter().find_map(|child| {
            if let Node::Block(block) = child {
                if block.name == Some(name) { Some(block) } else { None }
            } else {
                None
            }
        })
    }

    pub fn find_child_blocks(&self, name: Spur) -> Vec<&Block> {
        self.children
            .iter()
            .filter_map(|child| {
                if let Node::Block(block) = child {
                    if block.name == Some(name) { Some(block) } else { None }
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn has_attribute(&self, key: Spur) -> bool {
        self.attributes.contains_key(&key)
    }

    pub fn has_name(&self) -> bool {
        self.name.is_some()
    }
}

impl Value {
    pub fn as_string(&self) -> Option<&String> {
        if let Value::String(s) = self { Some(s) } else { None }
    }

    pub fn as_number(&self) -> Option<f64> {
        if let Value::Number(n) = self { Some(*n) } else { None }
    }

    pub fn as_boolean(&self) -> Option<bool> {
        if let Value::Boolean(b) = self { Some(*b) } else { None }
    }

    pub fn as_array(&self) -> Option<&Vec<Value>> {
        if let Value::Array(arr) = self { Some(arr) } else { None }
    }
}
