use crate::{
    Span,
    ast::{
        Block, MarstonDocument,
        ident_table::{get_or_intern, resolve},
    },
    info::Info,
    report,
    reports::ReportsBag,
    validator::{Label, Report, Validate, ValidationRule},
};
use ariadne::{Color, ReportKind};
use itertools::Itertools;
use std::collections::HashMap;

impl Validate for Block {
    fn rules() -> Vec<ValidationRule<Self>> {
        vec![validate_attributes_uniqueness]
    }

    fn validate(&self, info: &mut Info) {
        for rule in Self::rules() {
            rule(self, info);
        }
    }
}

pub fn validate_attributes_uniqueness(block: &Block, info: &mut Info) {
    let block_name = if let Some(name) = block.name.as_ref() {
        resolve(name.key)
    } else {
        "<error>".to_string()
    };
    let mut attr_spans: HashMap<String, Vec<Span>> = HashMap::new();

    for attr in &block.attributes {
        let name = resolve(attr.key.key);
        attr_spans.entry(name).or_default().push(attr.key.span.clone());
    }

    for (attr, spans) in attr_spans {
        if spans.len() > 1 {
            ReportsBag::add(report!(
                kind: ReportKind::Error,
                message: format!("Duplicate attribute '{}' found in '{block_name}'", attr),
                labels: {
                    spans.first().unwrap().clone() => format!("Attribute '{}' first defined here", attr) => Color::BrightRed
                },
                label_vec: spans.iter().skip(1).map(|s| (s.clone(), "Attribute redefined here")).collect_vec() => Color::Yellow,
            ))
        }
    }
}
