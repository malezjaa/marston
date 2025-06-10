use crate::ast::MarstonDocument;
use crate::validator::{Validate, ValidationRule};

impl Validate for MarstonDocument {
    fn rules() -> Vec<ValidationRule<Self>> {
        vec![]
    }
    
    fn validate(&self) {
        self.call_rules();
        
        for block in &self.blocks {
            block.validate();
        }
    }
}