use crate::{
    Span,
    ast::{Block, MarstonDocument, ident_table::get_or_intern},
    info::Info,
    report,
    reports::ReportsBag,
    validator::{GenericValidator, Label, Report, ValidUrlOptions},
};
use ariadne::{Color, ReportKind};
use mime::Mime;

pub fn validate_link(doc: &MarstonDocument, info: &mut Info) {
    GenericValidator::new("rel")
        .as_attribute()
        .must_be_string()
        .string_not_empty()
        .required()
        .in_parent(vec!["head", "link"])
        .string_allowed_values(&[
            "alternate",
            "dns-prefetch",
            "icon",
            "manifest",
            "modulepreload",
            "pingback",
            "preconnect",
            "prefetch",
            "preload",
            "prerender",
            "stylesheet",
        ])
        .validate(doc, info);

    GenericValidator::new("href")
        .in_parent(vec!["head", "link"])
        .required()
        .as_attribute()
        .must_be_string()
        .string_not_empty()
        .string_valid_url(Some(ValidUrlOptions::new(&[], true)))
        .string_prefer_https()
        .validate(doc, info);

    GenericValidator::new("type")
        .in_parent(vec!["head", "link"])
        .as_attribute()
        .must_be_string()
        .string_not_empty()
        .check_value(|val, span| {
            if let Some(val) = val.kind.as_string()
                && let Err(err) = val.parse::<Mime>()
            {
                ReportsBag::add(report!(
                    kind: ReportKind::Error,
                    message: format!("Found invalid MIME type: {val}. {err}"),
                    labels: {
                        span.clone() => "invalid MIME type" => Color::BrightRed
                    }
                ));
            }
        })
        .validate(doc, info);
}
