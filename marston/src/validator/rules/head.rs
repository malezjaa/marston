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
use std::{collections::HashMap, fmt::format, sync::Arc};

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
    if (info.no_head) {
        return;
    }

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
    if (info.no_head) {
        return;
    }
    GenericValidator::new("keywords")
        .as_attribute()
        .must_be_array(Some(ValueKind::dummy_string()))
        .in_parent(vec!["head"])
        .array_not_empty()
        .check_value(|value, span| {
            if let Some(arr) = value.kind.as_array() {
                for item in arr {
                    if let Some(s) = item.kind.as_string() && s.trim().is_empty() {
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
        })
        .validate(doc, info);
}

pub fn validate_charset(doc: &MarstonDocument, info: &mut Info) {
    if (info.no_head) {
        return;
    }

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
    if (info.no_head) {
        return;
    }

    GenericValidator::new("viewport")
        .as_attribute()
        .in_parent(vec!["head"])
        .must_be_string()
        .string_not_empty()
        .check_value(|value, span| {
            if let Some(s) = value.kind.as_string() && !s.trim().to_lowercase().contains("width=device-width") {
                ReportsBag::add(report!(
                    kind: ReportKind::Warning,
                    message: "Viewport should include 'width=device-width' for mobile compatibility".to_string(),
                    labels: {
                        span.clone() => "Consider adding 'width=device-width' to viewport" => Color::BrightYellow
                    },
                    notes: ["Example: 'width=device-width, initial-scale=1.0'"]
                ));
            }
        })
        .validate(doc, info);
}
