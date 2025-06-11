use crate::{
    ast::Block,
    validator::{Validate, ValidationRule},
};
use crate::info::Info;

impl Validate for Block {
    fn rules() -> Vec<ValidationRule<Self>> {
        vec![]
    }

    fn validate(&self, info: &mut Info) {}
}
