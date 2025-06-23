use crate::{
    Span,
    ast::{Block, MarstonDocument, ident_table::get_or_intern},
    info::Info,
    report,
    reports::ReportsBag,
    validator::{
        GenericValidator, Label, Report,
        conditions::{AttributeEquals, ConditionResult},
        url::{RequiredExtension, UrlValidation},
    },
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
        .string_allowed_values(
            &[
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
            ],
            false,
        )
        .validate(doc, info);

    GenericValidator::new("href")
        .in_parent(vec!["head", "link"])
        .required()
        .as_attribute()
        .must_be_string()
        .string_not_empty()
        .string_valid_url(Some(UrlValidation::new(
            &[],
            true,
            Some(RequiredExtension::new(
                "css",
                Box::new(AttributeEquals::new(|block| {
                    if let Some(attr) = block.get_attribute("rel") {
                        if let Some(string) = attr.value.kind.as_string() {
                            return ConditionResult::new(string == "stylesheet", Some("link elements with rel=\"stylesheet\" must have a file extension of .css"))
                        }
                    }

                    ConditionResult::new(false, None)
                })),
            )),
        )))
        .string_prefer_https()
        .validate(doc, info);

    GenericValidator::new("type")
        .in_parent(vec!["head", "link"])
        .as_attribute()
        .must_be_string()
        .string_not_empty()
        .check_value(|val, span, _| {
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

    GenericValidator::new("as")
        .in_parent(vec!["head", "link"])
        .as_attribute()
        .must_be_string()
        .string_not_empty()
        .required_if(AttributeEquals::new(|block| {
            if let Some(rel) = block.get_attribute("rel")
                && let Some(string) = rel.value.kind.as_string()
            {
                if string == "preload" {
                    return ConditionResult::new(
                        true,
                        Some("preload elements must have an as attribute"),
                    );
                }
            }

            ConditionResult::new(false, None)
        }))
        .string_allowed_values(
            &[
                "audio", "document", "embed", "fetch", "font", "image", "object", "script",
                "style", "track", "video", "worker",
            ],
            true,
        )
        .validate(doc, info);
}
