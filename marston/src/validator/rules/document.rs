use crate::{
    Span,
    ast::{Block, MarstonDocument, ident_table::resolve},
    error_report,
    html::tags::is_unique_tag,
    info::Info,
    reports::ReportsBag,
    validator::{Validate, ValidationRule},
};
use ariadne::{Color, Label, Report, ReportKind};
use itertools::Itertools;
use std::{collections::HashMap, sync::Arc};

impl Validate for MarstonDocument {
    fn rules() -> Vec<ValidationRule<Self>> {
        vec![validate_block_name_uniqueness]
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
        ReportsBag::add(error_report!(
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
