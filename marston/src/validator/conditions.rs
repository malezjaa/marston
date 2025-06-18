use crate::{
    Span,
    ast::{Block, Value},
    validator::GenericValidator,
};
use std::collections::HashMap;

pub trait Condition {
    fn evaluate(&self, context: &ValidationContext) -> bool;
}

pub struct ValidationContext<'a> {
    pub block: &'a Block,
}

impl<'a> ValidationContext<'a> {
    pub fn new(block: &'a Block) -> Self {
        ValidationContext { block }
    }
}

pub type AttributeEqualsPredicate = Box<dyn Fn(&Block) -> bool>;
pub struct AttributeEquals {
    pub predicate: AttributeEqualsPredicate,
}

impl AttributeEquals {
    pub fn new(predicate: fn(&Block) -> bool) -> Self {
        AttributeEquals { predicate: Box::new(predicate) }
    }
}

impl Condition for AttributeEquals {
    fn evaluate(&self, context: &ValidationContext) -> bool {
        (self.predicate)(context.block)
    }
}

pub struct And<C1: Condition, C2: Condition> {
    pub left: C1,
    pub right: C2,
}

impl<C1: Condition, C2: Condition> Condition for And<C1, C2> {
    fn evaluate(&self, ctx: &ValidationContext) -> bool {
        self.left.evaluate(ctx) && self.right.evaluate(ctx)
    }
}

impl GenericValidator {
    pub fn required_if(mut self, condition: impl Condition + 'static) -> Self {
        self.required_condition = Some(Box::new(condition));
        self
    }

    pub fn is_required(&self, context: &ValidationContext) -> bool {
        match &self.required_condition {
            Some(cond) => cond.evaluate(context),
            None => false,
        }
    }
}
