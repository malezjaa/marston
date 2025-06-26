use crate::{
    Span,
    ast::{Block, Value},
    validator::GenericValidator,
};
use std::{collections::HashMap, fmt::Debug};

pub struct ConditionResult {
    pub result: bool,
    pub messages: Vec<&'static str>,
}

impl ConditionResult {
    pub fn new(result: bool, message: Option<&'static str>) -> Self {
        ConditionResult {
            result,
            messages: if let Some(msg) = message { vec![msg] } else { vec![] },
        }
    }

    pub fn join(self, other: ConditionResult) -> Self {
        ConditionResult {
            result: self.result && other.result,
            messages: self.messages.into_iter().chain(other.messages).collect(),
        }
    }
}

pub trait Condition: Debug {
    fn evaluate(&self, context: &ValidationContext) -> ConditionResult;
}

pub struct ValidationContext<'a> {
    pub block: &'a Block,
}

impl<'a> ValidationContext<'a> {
    pub fn new(block: &'a Block) -> Self {
        ValidationContext { block }
    }
}

impl<'a> Into<ValidationContext<'a>> for &'a Block {
    fn into(self) -> ValidationContext<'a> {
        ValidationContext::new(self)
    }
}

pub trait AttributeEqualsPredicateTrait: Fn(&Block) -> ConditionResult + Debug {}
impl<T> AttributeEqualsPredicateTrait for T where T: Fn(&Block) -> ConditionResult + Debug {}

pub type AttributeEqualsPredicate = Box<dyn AttributeEqualsPredicateTrait>;

#[derive(Debug)]
pub struct AttributeEquals {
    pub predicate: AttributeEqualsPredicate,
}

impl AttributeEquals {
    pub fn new(predicate: fn(&Block) -> ConditionResult) -> Self {
        AttributeEquals { predicate: Box::new(predicate) }
    }
}

impl Condition for AttributeEquals {
    fn evaluate(&self, context: &ValidationContext) -> ConditionResult {
        (self.predicate)(context.block)
    }
}

#[derive(Debug)]
pub struct And<C1: Condition, C2: Condition> {
    pub left: C1,
    pub right: C2,
}

impl<C1: Condition, C2: Condition> Condition for And<C1, C2> {
    fn evaluate(&self, ctx: &ValidationContext) -> ConditionResult {
        self.left.evaluate(ctx).join(self.right.evaluate(ctx))
    }
}

impl GenericValidator {
    pub fn required_if(mut self, condition: impl Condition + 'static) -> Self {
        self.required_condition = Some(Box::new(condition));
        self
    }

    pub fn is_required(&self, context: &ValidationContext) -> bool {
        match &self.required_condition {
            Some(cond) => cond.evaluate(context).result,
            None => false,
        }
    }

    pub fn valid_if(mut self, condition: impl Condition + 'static) -> Self {
        self.valid_if = Some(Box::new(condition));
        self
    }
}
