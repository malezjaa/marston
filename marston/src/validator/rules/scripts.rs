use crate::{
    Span,
    ast::{Block, MarstonDocument, ident_table::get_or_intern},
    info::Info,
    validator::{
        GenericValidator, validate_block_no_children, validate_block_not_empty,
        validate_mutually_exclusive_attributes,
    },
};

pub fn validate_script(doc: &MarstonDocument, info: &mut Info) {
    if let Some(head_block) = doc.find_block_by_name(get_or_intern("head")) {
        let blocks = head_block.find_all_by_name(get_or_intern("script"));

        for script_block in blocks {
            validate_script_block(script_block, info, doc);
        }
    }
}

fn validate_script_block(script_block: &Block, info: &mut Info, doc: &MarstonDocument) {
    validate_block_no_children(script_block, "Script");

    validate_block_not_empty(script_block, "Script", Some("src"));

    GenericValidator::new("src")
        .as_attribute()
        .must_be_string()
        .in_parent(vec!["head", "script"])
        .string_not_empty()
        .string_valid_url(None)
        .string_file_extension(".js")
        .string_prefer_https()
        .validate(doc, info);

    GenericValidator::new("async")
        .in_parent(vec!["head", "script"])
        .as_attribute()
        .must_be_boolean()
        .validate(doc, info);

    GenericValidator::new("defer")
        .in_parent(vec!["head", "script"])
        .as_attribute()
        .must_be_boolean()
        .validate(doc, info);

    GenericValidator::new("type")
        .in_parent(vec!["head", "script"])
        .as_attribute()
        .must_be_string()
        .string_not_empty()
        .string_allowed_values(&["text/javascript", "application/javascript", "module"])
        .validate(doc, info);

    validate_mutually_exclusive_attributes(
        script_block,
        "async",
        "defer",
        "Script has both 'async' and 'defer' attributes",
        &[
            "When both are present, 'async' takes precedence",
            "Consider using only one loading strategy for clarity",
        ],
    );
}
