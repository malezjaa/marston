use crate::{
    Span,
    ast::ident_table::{get_or_intern, resolve},
};
use lasso::{Key, Spur};
use rustc_hash::FxHashMap;
use std::{
    fmt,
    fmt::{Display, Formatter},
    ops::Range,
};

pub mod ident_table;
pub mod parser;

#[derive(Debug, Clone)]
pub struct MarstonDocument {
    pub blocks: Vec<Block>,
}

impl MarstonDocument {
    pub fn find_in_parent(&self, names: &Vec<Spur>) -> Vec<&Block> {
        if names.is_empty() {
            return Vec::new();
        }

        let mut current_blocks: Vec<&Block> = Vec::new();

        let first_name = names[0];
        for block in &self.blocks {
            if let Some(ref interned) = block.name {
                if interned.key == first_name {
                    current_blocks.push(block);
                }
            }
        }

        for name in names.iter().skip(1) {
            let mut next_blocks = Vec::new();
            for block in current_blocks {
                for child in &block.children {
                    if let Node::Block(child_block) = child {
                        if let Some(ref interned) = child_block.name {
                            if interned.key == *name {
                                next_blocks.push(child_block);
                            }
                        }
                    }
                }
            }
            current_blocks = next_blocks;
        }

        current_blocks
    }
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
    pub id: usize,
    pub name: Option<Interned>,
    pub attributes: Vec<Attribute>,
    pub children: Vec<Node>,
    pub span: Span,
}

impl Block {
    pub fn find_all_by_name(&self, name: Spur) -> Vec<&Block> {
        let mut results = Vec::new();
        if let Some(n) = &self.name {
            if n.key == name {
                results.push(self);
            }
        }
        for child in &self.children {
            if let Node::Block(block) = child {
                results.extend(block.find_all_by_name(name));
            }
        }
        results
    }

    pub fn name(&self) -> Interned {
        self.name.as_ref().unwrap().clone()
    }

    pub fn find_all_child_blocks(&self, name: Spur) -> Vec<&Block> {
        let mut results = Vec::new();

        for child in &self.children {
            if let Node::Block(child) = child {
                if let Some(child_name) = &child.name
                    && child_name.key == name
                {
                    results.push(child);
                }
            }
        }

        results
    }
}

#[derive(Debug, Clone)]
pub enum ValueKindHelper {
    String,
    Bool,
    Array,
    Number,
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

impl Value {
    /// Creates a new `Value` with boolean true.
    pub fn new_default(span: Span) -> Self {
        Self { kind: ValueKind::Boolean(true), span }
    }
}

#[derive(Debug, Clone)]
pub enum ValueKind {
    String(String),
    Number(f64),
    Boolean(bool),
    Array(Vec<Value>),
}

impl ValueKind {
    /// Only checks the kind ignoring the inner value.
    pub fn is_same_kind(&self, other: &ValueKind) -> bool {
        match (self, other) {
            (ValueKind::String(_), ValueKind::String(_)) => true,
            (ValueKind::Number(_), ValueKind::Number(_)) => true,
            (ValueKind::Boolean(_), ValueKind::Boolean(_)) => true,
            (ValueKind::Array(_), ValueKind::Array(_)) => true,
            _ => false,
        }
    }

    pub fn dummy_string() -> Self {
        Self::String("".to_string())
    }
}

impl Display for ValueKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ValueKind::String(s) => write!(f, "{}", s),
            ValueKind::Number(n) => write!(f, "{}", n),
            ValueKind::Boolean(b) => write!(f, "{}", b),
            ValueKind::Array(arr) => write!(
                f,
                "[{}]",
                arr.iter().map(|v| v.kind.to_string()).collect::<Vec<_>>().join(", ")
            ),
        }
    }
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
}

impl Block {
    pub fn new(name: Option<Interned>, id: usize) -> Self {
        Self { name, attributes: Vec::new(), children: Vec::new(), span: Range::default(), id }
    }

    pub fn get_attribute(&self, key: &str) -> Option<&Attribute> {
        self.attributes.iter().find(|attr| attr.key.key == get_or_intern(key))
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
