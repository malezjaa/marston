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
    validator::{GenericValidator, Validate, ValidationRule, rules::scripts::validate_script},
};
use ariadne::{Color, Label, Report, ReportKind};
use itertools::Itertools;
use lasso::Spur;
use std::{collections::HashMap, sync::Arc};

impl Validate for MarstonDocument {
    fn rules() -> Vec<ValidationRule<Self>> {
        vec![
            ensure_required_tags,
            validate_block_name_uniqueness,
            validate_title,
            validate_lang,
            validate_charset,
            validate_viewport,
            validate_script,
            validate_keywords
        ]
    }

    fn validate(&self, info: &mut Info) {
        self.call_rules(info);

        for block in &self.blocks {
            block.validate(info);
        }
    }
}

pub fn validate_title(doc: &MarstonDocument, info: &mut Info) {
    GenericValidator::new("title")
        .as_attribute()
        .in_parent(vec!["head"])
        .required()
        .must_be_string()
        .string_not_empty()
        .string_min_length(10)
        .string_max_length(100)
        .string_not_generic(&[
            "home",
            "page",
            "index",
            "untitled",
            "document",
            "welcome",
            "default title",
            "new page",
            "title",
        ])
        .validate(doc, info);
}

pub fn validate_lang(doc: &MarstonDocument, info: &mut Info) {
    GenericValidator::new("lang")
        .as_attribute()
        .in_parent(vec!["head"])
        .required()
        .must_be_string()
        .string_not_empty()
        .string_valid_language_code()
        .validate(doc, info);
}

pub fn validate_keywords(doc: &MarstonDocument, info: &mut Info) {
    GenericValidator::new("keywords")
        .as_attribute()
        .must_be_array(Some(ValueKind::dummy_string()))
        .in_parent(vec!["head"])
        .array_not_empty()
        .check_value(|value, span| {
            if let Some(arr) = value.kind.as_array() {
                for item in arr {
                    if let Some(s) = item.kind.as_string() {
                        if s.trim().is_empty() {
                            ReportsBag::add(report!(
                                kind: ReportKind::Warning,
                                message: "Empty keyword found".to_string(),
                                labels: {
                                    item.span.clone() => "Keywords should not contain empty strings" => Color::BrightYellow
                                },
                                notes: ["Consider removing empty keywords from the list"]
                            ));
                        }
                    }
                }
            }
        })
        .validate(doc, info);
}

pub fn validate_charset(doc: &MarstonDocument, info: &mut Info) {
    GenericValidator::new("charset")
        .as_attribute()
        .in_parent(vec!["head"])
        .required()
        .must_be_string()
        .string_not_empty()
        .check_value(|value, span| {
            if let Some(s) = value.kind.as_string() {
                let normalized = s.trim().to_lowercase();
                if !["utf-8", "utf8", "iso-8859-1", "windows-1252"].contains(&normalized.as_str()) {
                    ReportsBag::add(report!(
                        kind: ReportKind::Warning,
                        message: "Uncommon character encoding".to_string(),
                        labels: {
                            span.clone() => "Consider using UTF-8 for better compatibility" => Color::BrightYellow
                        },
                        notes: ["UTF-8 is the recommended encoding for web documents"]
                    ));
                }
            }
        })
        .validate(doc, info);
}

pub fn validate_viewport(doc: &MarstonDocument, info: &mut Info) {
    GenericValidator::new("viewport")
        .as_attribute()
        .in_parent(vec!["head"])
        .must_be_string()
        .string_not_empty()
        .check_value(|value, span| {
            if let Some(s) = value.kind.as_string() {
                let lower = s.trim().to_lowercase();
                if !lower.contains("width=device-width") {
                    ReportsBag::add(report!(
                        kind: ReportKind::Warning,
                        message: "Viewport should include 'width=device-width' for mobile compatibility".to_string(),
                        labels: {
                            span.clone() => "Consider adding 'width=device-width' to viewport" => Color::BrightYellow
                        },
                        notes: ["Example: 'width=device-width, initial-scale=1.0'"]
                    ));
                }
            }
        })
        .validate(doc, info);
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
