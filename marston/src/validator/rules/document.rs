use crate::{
    Span,
    ast::{
        Attribute, Block, MarstonDocument, Value, ValueKind,
        ident_table::{get_or_intern, resolve},
    },
    html::tags::is_unique_tag,
    info::{BlockInfo, Info},
    report,
    reports::ReportsBag,
    validator::{
        GenericValidator, Validate, ValidationRule,
        rules::{
            head::{
                validate_base, validate_charset, validate_keywords, validate_lang, validate_title,
                validate_viewport,
            },
            scripts::validate_script,
        },
    },
};
use ariadne::{Color, Label, Report, ReportKind};
use itertools::Itertools;
use lasso::Spur;
use std::{collections::HashMap, fmt::format, sync::Arc};
use crate::validator::rules::link::validate_link;

impl Validate for MarstonDocument {
    fn rules() -> Vec<ValidationRule<Self>> {
        vec![
            ensure_required_tags,
            validate_block_name_uniqueness,
            validate_lang,
            validate_charset,
            validate_title,
            validate_viewport,
            validate_keywords,
            validate_script,
            validate_base,
            validate_link
        ]
    }

    fn validate(&self, info: &mut Info) {
        self.call_rules(info);

        for block in &self.blocks {
            block.validate(info);
        }
    }
}

pub fn validate_block_name_uniqueness(_: &MarstonDocument, info: &mut Info) {
    let mut seen: HashMap<String, &BlockInfo> = HashMap::new();
    let mut duplicates = vec![];

    for block in info.blocks() {
        let name_str = resolve(block.name.key);
        if let Some(existing) = seen.get(&name_str) {
            duplicates.push(((name_str.clone(), block.span.clone()), existing.span.clone()));
        } else {
            seen.insert(name_str, block);
        }
    }

    for ((name, dup_span), orig_span) in duplicates {
        if (!is_unique_tag(name.as_str())) {
            continue;
        }

        ReportsBag::add(report!(
            kind: ReportKind::Error,
            message: format!("Duplicate block name '{}' found", name),
            labels: {
                dup_span => format!("Block '{}' redefined here", name) => Color::BrightRed,
                orig_span => format!("Block '{}' first defined here", name) => Color::Yellow
            },
            notes: [format!("'{name}' is a block that should occur only once per document")]
        ));
    }
}

pub fn ensure_required_tags(doc: &MarstonDocument, info: &mut Info) {
    let required_tags = ["head", "body"];
    let mut missing_tags = vec![];

    for tag in required_tags {
        if !info.has_block(get_or_intern(tag)) {
            if tag == "head" {
                info.no_head = true;
            }

            missing_tags.push(tag);
        }
    }

    if !missing_tags.is_empty() {
        ReportsBag::add(report!(
            kind: ReportKind::Error,
            message: format!("Missing required tags: {}", missing_tags.iter().join(", ")),
            labels: {
                Span::default() => "These tags are required for a valid Marston document" => Color::BrightRed
            },
            notes: ["Ensure that your document includes all required tags in its root"]
        ));
    }
}
