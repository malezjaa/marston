use crate::{
    ast::Block,
    info::Info,
    validator::{Validate, ValidationRule},
};

impl Validate for Block {
    fn rules() -> Vec<ValidationRule<Self>> {
        vec![]
    }

    fn validate(&self, info: &mut Info) {}
}
