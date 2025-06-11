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

#[derive(Clone, Debug)]
pub struct Interned {
    pub span: Span,
    pub key: Spur,
}

impl Interned {
    pub fn new(name: Spur, span: Span) -> Self {
        Self { key: name, span }
    }
}

#[derive(Debug, Clone)]
pub struct Block {
    pub name: Option<Interned>,
    pub attributes: Vec<Attribute>,
    pub children: Vec<Node>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Attribute {
    pub key: Interned,
    pub value: Value,
}

impl Attribute {
    pub fn new(key: Interned, value: Value) -> Self {
        Self { key, value }
    }
}

#[derive(Debug, Clone)]
pub struct Value {
    pub kind: ValueKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ValueKind {
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
        self.blocks.iter().find(|e| e.name.as_ref().map(|n| n.key) == Some(name))
    }

    pub fn find_blocks_by_name(&self, name: Spur) -> Vec<&Block> {
        self.blocks.iter().filter(|e| e.name.as_ref().map(|n| n.key) == Some(name)).collect()
    }
}

impl Block {
    pub fn new(name: Option<Interned>) -> Self {
        Self { name, attributes: Vec::new(), children: Vec::new(), span: Default::default() }
    }

    pub fn with_attributes(name: Option<Interned>, attributes: Vec<Attribute>) -> Self {
        Self { name, attributes, children: Vec::new(), span: Default::default() }
    }

    pub fn add_attribute(&mut self, key: Interned, value: Value) {
        self.attributes.push(Attribute { key, value });
    }

    pub fn get_attribute(&self, key: Spur) -> Option<&Value> {
        self.attributes.iter().find(|attr| attr.key.key == key).map(|attr| &attr.value)
    }

    pub fn has_attribute(&self, key: Spur) -> bool {
        self.attributes.iter().any(|attr| attr.key.key == key)
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
                if block.name.as_ref().map(|n| n.key) == Some(name) { Some(block) } else { None }
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
                    if block.name.as_ref().map(|n| n.key) == Some(name) {
                        Some(block)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn has_name(&self) -> bool {
        self.name.is_some()
    }
}

impl ValueKind {
    pub fn as_string(&self) -> Option<&String> {
        if let ValueKind::String(s) = self { Some(s) } else { None }
    }

    pub fn as_number(&self) -> Option<f64> {
        if let ValueKind::Number(n) = self { Some(*n) } else { None }
    }

    pub fn as_boolean(&self) -> Option<bool> {
        if let ValueKind::Boolean(b) = self { Some(*b) } else { None }
    }

    pub fn as_array(&self) -> Option<&Vec<Value>> {
        if let ValueKind::Array(arr) = self { Some(arr) } else { None }
    }
}
