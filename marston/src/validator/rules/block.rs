use crate::{
    ast::Block,
    validator::{Validate, ValidationRule},
};

impl Validate for Block {
    fn rules() -> Vec<ValidationRule<Self>> {
        vec![]
    }

    fn validate(&self) {}
}
