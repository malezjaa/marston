use lasso::{Key, Spur};
use std::collections::HashMap;
use crate::Span;

pub mod ident_table;

#[derive(Debug, Clone)]
pub struct MarstonDocument {
    pub elements: Vec<Element>,
}

#[derive(Debug, Clone)]
pub enum Node {
    Element(Element),
    Text(String),
    Comment(String),
}

#[derive(Debug, Clone)]
pub struct Element {
    pub name: Spur,
    pub attributes: HashMap<Spur, AttributeValue>,
    pub children: Vec<Node>,
    pub span: Option<Span>,
}

#[derive(Debug, Clone)]
pub enum AttributeValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Array(Vec<AttributeValue>),
}

impl MarstonDocument {
    pub fn new() -> Self {
        Self { elements: Vec::new() }
    }

    pub fn add_element(&mut self, element: Element) {
        self.elements.push(element);
    }

    pub fn find_element_by_name(&self, name: Spur) -> Option<&Element> {
        self.elements.iter().find(|e| e.name == name)
    }

    pub fn find_elements_by_name(&self, name: Spur) -> Vec<&Element> {
        self.elements.iter().filter(|e| e.name == name).collect()
    }
}

impl Element {
    pub fn new(name: Spur) -> Self {
        Self { name, attributes: HashMap::new(), children: Vec::new(), span: None }
    }

    pub fn with_attributes(name: Spur, attributes: HashMap<Spur, AttributeValue>) -> Self {
        Self { name, attributes, children: Vec::new(), span: None }
    }

    pub fn add_attribute(&mut self, key: Spur, value: AttributeValue) {
        self.attributes.insert(key, value);
    }

    pub fn get_attribute(&self, key: Spur) -> Option<&AttributeValue> {
        self.attributes.get(&key)
    }

    pub fn add_child(&mut self, child: Node) {
        self.children.push(child);
    }

    pub fn add_text(&mut self, text: String) {
        self.children.push(Node::Text(text));
    }

    pub fn add_element(&mut self, element: Element) {
        self.children.push(Node::Element(element));
    }

    pub fn find_child_element(&self, name: Spur) -> Option<&Element> {
        self.children.iter().find_map(|child| {
            if let Node::Element(element) = child {
                if element.name == name { Some(element) } else { None }
            } else {
                None
            }
        })
    }

    pub fn find_child_elements(&self, name: Spur) -> Vec<&Element> {
        self.children
            .iter()
            .filter_map(|child| {
                if let Node::Element(element) = child {
                    if element.name == name { Some(element) } else { None }
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn has_attribute(&self, key: Spur) -> bool {
        self.attributes.contains_key(&key)
    }
}

impl AttributeValue {
    pub fn as_string(&self) -> Option<&String> {
        if let AttributeValue::String(s) = self { Some(s) } else { None }
    }

    pub fn as_number(&self) -> Option<f64> {
        if let AttributeValue::Number(n) = self { Some(*n) } else { None }
    }

    pub fn as_boolean(&self) -> Option<bool> {
        if let AttributeValue::Boolean(b) = self { Some(*b) } else { None }
    }

    pub fn as_array(&self) -> Option<&Vec<AttributeValue>> {
        if let AttributeValue::Array(arr) = self { Some(arr) } else { None }
    }
}
