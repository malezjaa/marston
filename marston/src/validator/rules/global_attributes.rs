use crate::{ast::MarstonDocument, info::Info, validator::GenericValidator};

// global attributes apply to every block
pub fn validate_global_attributes(doc: &MarstonDocument, info: &mut Info) {
    GenericValidator::new("autocapitalize")
        .as_attribute()
        .validate_all()
        .string_not_empty()
        .string_allowed_values(&["none", "off", "sentences", "on", "words", "characters"], true)
        .validate(doc, info);
}
