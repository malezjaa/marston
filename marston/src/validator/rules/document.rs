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
    let all_names: Vec<(String, Span)> =
        info.blocks.iter().map(|block| (resolve(block.name.key), block.span.clone())).collect();

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
            span: Span::default(),
            message: format!("Missing required tags: {}", missing_tags.iter().join(", ")),
            labels: {
                Span::default() => "These tags are required for a valid Marston document" => Color::BrightRed
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
                    head.span.clone() => "The 'title' block is required within the 'head' tag" => Color::BrightRed
                },
                notes: ["Title attribute has to be direct child of 'head' block"]
            ));
        }

        if let Some(title) = head.find_child_block(get_or_intern("title")) {
            let interned = title.name.clone().unwrap();
            ReportsBag::add(report!(
                kind: ReportKind::Error,
                span: Default::default(),
                message: "Title should be specified as an attribute, not a block".to_string(),
                labels: {
                    interned.span => "Title block found, but it should be an attribute" => Color::BrightRed
                },
                notes: ["Example: .title = 'My Document Title'"]
            ))
        }
    }
}

pub fn validate_title(doc: &MarstonDocument, info: &mut Info) {
    if let Some(head) = doc.find_block_by_name(get_or_intern("head")) {
        if let Some(title) = head.get_attribute(get_or_intern("title")) {
            if let Some(string) = title.kind.as_string() {
                let trimmed = string.trim();

                if trimmed.is_empty() {
                    ReportsBag::add(report!(
                        kind: ReportKind::Error,
                        span: title.span.clone(),
                        message: "Title cannot be empty".to_string(),
                        labels: {
                            title.span.clone() =>  "The 'title' attribute must have a non-empty value" => Color::BrightRed
                        },
                        notes: ["A valid title is essential for document identification and SEO"]
                    ));
                    return;
                }

                if trimmed.len() < 10 {
                    ReportsBag::add(report!(
                        kind: ReportKind::Warning,
                        span: title.span.clone(),
                        message: "Title is very short".to_string(),
                        labels: {
                            title.span.clone() => "Consider making the title more descriptive" => Color::BrightYellow
                        },
                        notes: ["Very short titles may not provide enough context to users or search engines"]
                    ));
                } else if trimmed.len() > 100 {
                    ReportsBag::add(report!(
                        kind: ReportKind::Warning,
                        span: title.span.clone(),
                        message: "Title length exceeds 100 characters".to_string(),
                        labels: {
                            title.span.clone() => "Consider shortening the title" => Color::BrightYellow
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
                        span: title.span.clone(),
                        message: "Generic or uninformative title".to_string(),
                        labels: {
                            title.span.clone() => "Try using a more specific and descriptive title" => Color::BrightYellow
                        },
                        notes: ["Titles like 'Home' or 'Untitled' provide little context or SEO value"]
                    ));
                }

                if trimmed.chars().all(|c| !c.is_alphanumeric()) {
                    ReportsBag::add(report!(
                        kind: ReportKind::Warning,
                        span: title.span.clone(),
                        message: "Title is made up entirely of non-alphanumeric characters".to_string(),
                        labels: {
                            title.span.clone() =>  "Use meaningful words in the title" => Color::BrightYellow
                        },
                        notes: ["Avoid using only symbols or punctuation in titles"]
                    ));
                }
            } else {
                ReportsBag::add(report!(
                    kind: ReportKind::Error,
                    span: title.span.clone(),
                    message: "Title must be a string".to_string(),
                    labels: {
                        title.span.clone() => "The 'title' attribute should be a string value" => Color::BrightRed
                    },
                    notes: ["Ensure the title is defined as a string, e.g., .title = 'My Title'"]
                ));
            }
        }
    }
}
