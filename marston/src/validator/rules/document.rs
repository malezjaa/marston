use crate::{
    Span,
    ast::{
        Block, MarstonDocument,
        ident_table::{get_or_intern, resolve},
    },
    html::tags::is_unique_tag,
    info::{BlockInfo, Info},
    report,
    reports::ReportsBag,
    validator::{Validate, ValidationRule},
};
use ariadne::{Color, Label, Report, ReportKind};
use itertools::Itertools;
use std::{collections::HashMap, sync::Arc};

impl Validate for MarstonDocument {
    fn rules() -> Vec<ValidationRule<Self>> {
        vec![ensure_required_tags, validate_block_name_uniqueness, validate_title]
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

    if !missing_tags.contains(&"head") {
        let head = doc.find_block_by_name(get_or_intern("head")).unwrap();

        validate_title_type_and_hierarchy(head);
    }
}

fn validate_title_type_and_hierarchy(head: &Block) {
    let name_span = if let Some(name) = &head.name { name.span.clone() } else { Span::default() };

    if head.get_attribute(get_or_intern("title")).is_none() {
        ReportsBag::add(report!(
            kind: ReportKind::Error,
            message: "Missing 'title' attribute in 'head'".to_string(),
            labels: {
                name_span.clone() => "The 'title' attribute is required within the 'head' block" => Color::BrightRed
            },
            notes: ["Title attribute has to be direct child of 'head' block"]
        ));
    }

    if let Some(_) = head.find_child_block(get_or_intern("title")) {
        ReportsBag::add(report!(
            kind: ReportKind::Error,
            message: "Title should be specified as an attribute, not a block".to_string(),
            labels: {
                name_span => "Title block found, but it should be an attribute" => Color::BrightRed
            },
            notes: ["Example: .title = 'My Document Title'"]
        ))
    }
}

pub fn validate_title(doc: &MarstonDocument, info: &mut Info) {
    if let Some(head) = doc.find_block_by_name(get_or_intern("head")) {
        if let Some(title) = head.get_attribute(get_or_intern("title")) {
            let title_span = title.value.span.clone();

            if let Some(string) = title.value.kind.as_string() {
                let trimmed = string.trim();

                if trimmed.is_empty() {
                    ReportsBag::add(report!(
                        kind: ReportKind::Error,
                        message: "Title cannot be empty".to_string(),
                        labels: {
                            title_span =>  "The 'title' attribute must have a non-empty value" => Color::BrightRed
                        },
                        notes: ["A valid title is essential for document identification and SEO"]
                    ));
                    return;
                }

                if trimmed.len() < 10 {
                    ReportsBag::add(report!(
                        kind: ReportKind::Warning,
                        message: "Title is very short".to_string(),
                        labels: {
                            title_span.clone() => "Consider making the title more descriptive" => Color::BrightYellow
                        },
                        notes: ["Very short titles may not provide enough context to users or search engines"]
                    ));
                } else if trimmed.len() > 100 {
                    ReportsBag::add(report!(
                        kind: ReportKind::Warning,
                        message: "Title length exceeds 100 characters".to_string(),
                        labels: {
                            title_span.clone() => "Consider shortening the title" => Color::BrightYellow
                        },
                        notes: ["A shorter title improves readability and SEO"]
                    ));
                }

                let lower = trimmed.to_lowercase();
                let generic_titles = [
                    "home",
                    "page",
                    "index",
                    "untitled",
                    "document",
                    "welcome",
                    "default title",
                    "new page",
                    "title",
                ];

                if generic_titles.iter().any(|t| lower == *t) {
                    ReportsBag::add(report!(
                        kind: ReportKind::Warning,
                        message: "Generic or uninformative title".to_string(),
                        labels: {
                            title_span.clone() => "Try using a more specific and descriptive title" => Color::BrightYellow
                        },
                        notes: ["Titles like 'Home' or 'Untitled' provide little context or SEO value"]
                    ));
                }

                if trimmed.chars().all(|c| !c.is_alphanumeric()) {
                    ReportsBag::add(report!(
                        kind: ReportKind::Warning,
                        message: "Title is made up entirely of non-alphanumeric characters".to_string(),
                        labels: {
                            title_span =>  "Use meaningful words in the title" => Color::BrightYellow
                        },
                        notes: ["Avoid using only symbols or punctuation in titles"]
                    ));
                }
            } else {
                ReportsBag::add(report!(
                    kind: ReportKind::Error,
                    message: "Title must be a string".to_string(),
                    labels: {
                        title_span => "The 'title' attribute should be a string value" => Color::BrightRed
                    },
                    notes: ["Ensure the title is defined as a string, e.g., .title = 'My Title'"]
                ));
            }
        }
    }
}
