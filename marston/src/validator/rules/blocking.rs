use crate::validator::GenericValidator;

pub fn blocking_attribute() -> GenericValidator {
    GenericValidator::new("blocking")
        .must_be_string()
        .as_attribute()
        .string_not_empty()
}