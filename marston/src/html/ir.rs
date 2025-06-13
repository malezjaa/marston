use crate::ast::{Attribute, Block, MarstonDocument, Node, ValueKind, ident_table::get_or_intern};
use lasso::Spur;

pub struct IrDoc {
    pub root: Vec<IrNode>,
}

pub enum IrNode {
    Element(IrElement),
    Text(String),
}

pub struct IrElement {
    pub tag: Spur,
    pub attributes: Vec<IrAttribute>,
    pub children: Vec<IrNode>,
}

impl IrElement {
    pub fn new(spur: Spur) -> Self {
        IrElement { tag: spur, attributes: vec![], children: vec![] }
    }

    pub fn get_attribute(&self, key: Spur) -> Option<&IrAttribute> {
        self.attributes.iter().find(|attr| attr.key == key)
    }
}

pub struct IrAttribute {
    pub key: Spur,
    pub value: ValueKind,
}

pub struct IrTransformBuilder {
    transformations: Vec<Box<dyn IrTransformation>>,
}

impl IrTransformBuilder {
    pub fn new() -> Self {
        Self { transformations: vec![] }
    }

    pub fn add_transformation(mut self, transformation: Box<dyn IrTransformation>) -> Self {
        self.transformations.push(transformation);
        self
    }

    pub fn attribute_to_tag<F>(
        self,
        source_tag: &str,
        attr_key: &str,
        target_tag: &str,
        value_mapper: F,
    ) -> Self
    where
        F: Fn(&ValueKind) -> Option<Vec<IrNode>> + 'static,
    {
        self.add_transformation(Box::new(AttributeToTagTransform {
            source_tag: get_or_intern(source_tag),
            attr_key: get_or_intern(attr_key),
            target_tag: get_or_intern(target_tag),
            value_mapper: Box::new(value_mapper),
        }))
    }

    pub fn move_attribute(self, from_tag: &str, to_tag: &str, attr_key: &str) -> Self {
        self.add_transformation(Box::new(MoveAttributeTransform {
            from_tag: get_or_intern(from_tag),
            to_tag: get_or_intern(to_tag),
            attr_key: get_or_intern(attr_key),
        }))
    }

    pub fn remove_attribute(self, tag: &str, attr_key: &str) -> Self {
        self.add_transformation(Box::new(RemoveAttributeTransform {
            tag: get_or_intern(tag),
            attr_key: get_or_intern(attr_key),
        }))
    }

    pub fn attribute_to_element(self, source_tag: &str, attr_key: &str, target_tag: &str) -> Self {
        self.add_transformation(Box::new(AttributeToElementTransform {
            source_tag: get_or_intern(source_tag),
            attr_key: get_or_intern(attr_key),
            target_tag: get_or_intern(target_tag),
        }))
    }

    pub fn attribute_to_meta_tag(self, source_tag: &str, attr_key: &str, meta_name: &str) -> Self {
        self.add_transformation(Box::new(AttributeToMetaTransform {
            source_tag: get_or_intern(source_tag),
            attr_key: get_or_intern(attr_key),
            meta_name: get_or_intern(meta_name),
        }))
    }

    pub fn apply(&self, element: &mut IrElement) {
        for transformation in &self.transformations {
            transformation.apply(element);
        }
    }
}

pub trait IrTransformation {
    fn apply(&self, element: &mut IrElement);
}

pub struct AttributeToTagTransform {
    source_tag: Spur,
    attr_key: Spur,
    target_tag: Spur,
    value_mapper: Box<dyn Fn(&ValueKind) -> Option<Vec<IrNode>>>,
}

impl IrTransformation for AttributeToTagTransform {
    fn apply(&self, element: &mut IrElement) {
        self.apply_recursive(element);
    }
}

impl AttributeToTagTransform {
    fn apply_recursive(&self, element: &mut IrElement) {
        if element.tag == self.source_tag {
            if let Some(attr) = element.get_attribute(self.attr_key) {
                if let Some(children) = (self.value_mapper)(&attr.value) {
                    let mut new_element = IrElement::new(self.target_tag);
                    new_element.children = children;
                    element.children.push(IrNode::Element(new_element));
                }
            }
        }

        for child in &mut element.children {
            if let IrNode::Element(child_element) = child {
                self.apply_recursive(child_element);
            }
        }
    }
}

pub struct MoveAttributeTransform {
    from_tag: Spur,
    to_tag: Spur,
    attr_key: Spur,
}

impl IrTransformation for MoveAttributeTransform {
    fn apply(&self, element: &mut IrElement) {
        self.apply_recursive(element);
    }
}

impl MoveAttributeTransform {
    fn apply_recursive(&self, element: &mut IrElement) {
        let mut attr_to_move = None;

        if element.tag == self.from_tag {
            if let Some(pos) = element.attributes.iter().position(|attr| attr.key == self.attr_key)
            {
                attr_to_move = Some(element.attributes.remove(pos));
            }
        }

        if let Some(attr) = attr_to_move {
            if let Some(target_element) = self.find_element_mut(element, self.to_tag) {
                target_element.attributes.push(attr);
            }
        }

        for child in &mut element.children {
            if let IrNode::Element(child_element) = child {
                self.apply_recursive(child_element);
            }
        }
    }

    fn find_element_mut<'a>(
        &self,
        element: &'a mut IrElement,
        tag: Spur,
    ) -> Option<&'a mut IrElement> {
        if element.tag == tag {
            return Some(element);
        }

        for child in &mut element.children {
            if let IrNode::Element(child_element) = child {
                if let Some(found) = self.find_element_mut(child_element, tag) {
                    return Some(found);
                }
            }
        }
        None
    }
}

pub struct AttributeToElementTransform {
    source_tag: Spur,
    attr_key: Spur,
    target_tag: Spur,
}

impl IrTransformation for AttributeToElementTransform {
    fn apply(&self, element: &mut IrElement) {
        self.apply_recursive(element);
    }
}

impl AttributeToElementTransform {
    fn apply_recursive(&self, element: &mut IrElement) {
        if element.tag == self.source_tag {
            if let Some(attr) = element.get_attribute(self.attr_key) {
                let mut new_element = IrElement::new(self.target_tag);
                new_element
                    .attributes
                    .push(IrAttribute { key: self.attr_key, value: attr.value.clone() });
                element.children.push(IrNode::Element(new_element));
            }
        }

        for child in &mut element.children {
            if let IrNode::Element(child_element) = child {
                self.apply_recursive(child_element);
            }
        }
    }
}

pub struct AttributeToMetaTransform {
    source_tag: Spur,
    attr_key: Spur,
    meta_name: Spur,
}

impl IrTransformation for AttributeToMetaTransform {
    fn apply(&self, element: &mut IrElement) {
        self.apply_recursive(element);
    }
}

impl AttributeToMetaTransform {
    fn apply_recursive(&self, element: &mut IrElement) {
        if element.tag == self.source_tag {
            if let Some(attr) = element.get_attribute(self.attr_key) {
                let mut new_element = IrElement::new(get_or_intern("meta"));
                new_element.attributes.push(IrAttribute {
                    key: get_or_intern("name"),
                    value: ValueKind::String(
                        crate::ast::ident_table::resolve(self.meta_name).to_string(),
                    ),
                });
                new_element
                    .attributes
                    .push(IrAttribute { key: get_or_intern("content"), value: attr.value.clone() });
                element.children.push(IrNode::Element(new_element));
            }
        }

        for child in &mut element.children {
            if let IrNode::Element(child_element) = child {
                self.apply_recursive(child_element);
            }
        }
    }
}

pub struct RemoveAttributeTransform {
    tag: Spur,
    attr_key: Spur,
}

impl IrTransformation for RemoveAttributeTransform {
    fn apply(&self, element: &mut IrElement) {
        self.apply_recursive(element);
    }
}

impl RemoveAttributeTransform {
    fn apply_recursive(&self, element: &mut IrElement) {
        if element.tag == self.tag {
            element.attributes.retain(|attr| attr.key != self.attr_key);
        }

        for child in &mut element.children {
            if let IrNode::Element(child_element) = child {
                self.apply_recursive(child_element);
            }
        }
    }
}

fn find_element(elements: &mut [IrNode], tag: Spur) -> &mut IrElement {
    elements
        .iter_mut()
        .find_map(|node| match node {
            IrNode::Element(element) if element.tag == tag => Some(element),
            _ => None,
        })
        .unwrap()
}

pub trait ToHtmlIR {
    fn to_html_ir(&self) -> IrDoc;
}

impl ToHtmlIR for MarstonDocument {
    fn to_html_ir(&self) -> IrDoc {
        let mut root = IrElement::new(get_or_intern("html"));
        let elements = self.blocks.iter().map(|b| IrNode::Element(b.to_element())).collect();
        root.children = elements;

        let transformer = IrTransformBuilder::new()
            .move_attribute("head", "html", "lang")
            .attribute_to_tag("head", "title", "title", |value| {
                Some(vec![IrNode::Text(value.as_string().unwrap_or(&"".to_string()).clone())])
            })
            .remove_attribute("head", "title")
            .attribute_to_element("head", "charset", "meta")
            .remove_attribute("head", "charset")
            .attribute_to_meta_tag("head", "viewport", "viewport")
            .remove_attribute("head", "viewport")
            .attribute_to_meta_tag("head", "description", "description");

        transformer.apply(&mut root);

        IrDoc { root: vec![IrNode::Element(root)] }
    }
}

impl Block {
    fn to_element(&self) -> IrElement {
        IrElement {
            tag: self.name.as_ref().map(|n| n.key).unwrap_or_default(),
            attributes: self.attributes.iter().map(|a| a.to_html_attr()).collect(),
            children: self.children.iter().map(|n| n.to_html_node()).collect(),
        }
    }
}

impl Node {
    fn to_html_node(&self) -> IrNode {
        match self {
            Node::Block(block) => IrNode::Element(block.to_element()),
            Node::Text(text) => IrNode::Text(text.clone()),
        }
    }
}

impl Attribute {
    fn to_html_attr(&self) -> IrAttribute {
        IrAttribute { key: self.key.key, value: self.value.kind.clone() }
    }
}
