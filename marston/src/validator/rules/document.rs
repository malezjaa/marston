use crate::{
    Span,
    ast::{
        Block, MarstonDocument,
        ident_table::{get_or_intern, resolve},
    },
    html::tags::is_unique_tag,
    info::Info,
    report,
    reports::ReportsBag,
    validator::{Validate, ValidationRule},
};
use ariadne::{Color, Label, Report, ReportKind};
use itertools::Itertools;
use std::{collections::HashMap, sync::Arc};

impl Validate for MarstonDocument {
    fn rules() -> Vec<ValidationRule<Self>> {
        vec![ensure_required_tags, validate_block_name_uniqueness]
    }

    fn validate(&self, info: &mut Info) {
        self.call_rules(info);

        for block in &self.blocks {
            block.validate(info);
        }
    }
}

pub fn validate_block_name_uniqueness(_: &MarstonDocument, info: &mut Info) {
    let all_names: Vec<(String, Span)> =
        info.blocks.iter().map(|block| (resolve(block.name.clone()), block.span.clone())).collect();

    let mut seen: HashMap<String, Span> = HashMap::new();
    let mut duplicates = vec![];

    for (name, span) in &all_names {
        if let Some(existing_span) = seen.get(name) {
            duplicates.push(((name.clone(), span.clone()), existing_span.clone()));
        } else {
            seen.insert(name.clone(), span.clone());
        }
    }

    for ((name, dup_span), orig_span) in duplicates {
        ReportsBag::add(report!(
            kind: ReportKind::Error,
            span: dup_span.clone(),
            message: format!("Duplicate block name '{}' found", name),
            labels: {
                dup_span => {
                    message: format!("Block '{}' redefined here", name) => Color::BrightRed
                },
                orig_span => {
                    message: format!("Block '{}' first defined here", name) => Color::Yellow
                }
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
            missing_tags.push(tag);
        }
    }

    if !missing_tags.is_empty() {
        ReportsBag::add(report!(
            kind: ReportKind::Error,
            span: Span::default(),
            message: format!("Missing required tags: {}", missing_tags.iter().join(", ")),
            labels: {
                Span::default() => {
                    message: "These tags are required for a valid Marston document" => Color::BrightRed
                }
            },
            notes: ["Ensure that your document includes all required tags in its root"]
        ));
    }

    if !missing_tags.contains(&"head") {
        let head = doc.find_block_by_name(get_or_intern("head")).unwrap();

        if head.get_attribute(get_or_intern("title")).is_none() {
            ReportsBag::add(report!(
                kind: ReportKind::Error,
                span: head.span.clone(),
                message: "Missing 'title' attribute in 'head'".to_string(),
                labels: {
                    head.span.clone() => {
                        message: "The 'title' block is required within the 'head' tag" => Color::BrightRed
                    }
                },
                notes: ["Title attribute has to be direct child of 'head' block"]
            ));
        }
    }
}
