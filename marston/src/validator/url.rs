use crate::{
    Span,
    ast::Block,
    report,
    reports::ReportsBag,
    validator::{
        GenericValidator, Label, Report,
        conditions::{Condition, ValidationContext},
    },
};
use ariadne::{Color, ReportKind};
use std::path::{Path, PathBuf};
use url::Url;

#[derive(Default)]
pub struct UrlValidation {
    pub disallowed_protocols: &'static [&'static str],
    pub allow_paths: bool,
    pub required_extension: Option<RequiredExtension>,
}

impl UrlValidation {
    pub fn new(
        disallowed_protocols: &'static [&'static str],
        allow_paths: bool,
        required_extension: Option<RequiredExtension>,
    ) -> Self {
        Self { disallowed_protocols, allow_paths, required_extension }
    }
}

#[derive(Debug)]
pub struct RequiredExtension {
    pub extension: &'static str,
    pub condition: Box<dyn Condition>,
}

impl RequiredExtension {
    pub fn new(extension: &'static str, condition: Box<dyn Condition>) -> Self {
        Self { extension, condition }
    }
}

impl GenericValidator {
    fn validate_extension(path: &str, options: &UrlValidation, span: Span, block: &Block) {
        if let Some(ext) = &options.required_extension {
            let extension = ext.extension;

            let result = ext.condition.evaluate(&block.into());
            if result.result {
                let file_extension = Path::new(path).extension().and_then(|e| e.to_str());

                if file_extension != Some(extension) {
                    ReportsBag::add(report!(
                        kind: ReportKind::Error,
                        message: format!("URL should have '.{}' extension. {}", extension, result.messages.first().unwrap()),
                        labels: {
                            span.clone() => "invalid URL extension" => Color::BrightRed
                        },
                    ));
                }
            }
        }
    }

    pub fn string_valid_url(self, options: Option<UrlValidation>) -> Self {
        let options = options.unwrap_or_default();

        self.check_value(move |value, span, block| {
            if let Some(s) = value.kind.as_string() {
                let trimmed = s.trim();

                match Url::parse(trimmed) {
                    Ok(url) => {
                        if options.disallowed_protocols.iter().any(|&p| p == url.scheme()) {
                            ReportsBag::add(report!(
                                kind: ReportKind::Error,
                                message: format!("Found disallowed URL protocol: {}.", url.scheme()),
                                labels: {
                                    span.clone() => "disallowed invalid URL" => Color::BrightRed
                                },
                            ));
                        }

                        Self::validate_extension(url.path(), &options, span.clone(), block);
                    }
                    Err(err) => {
                        if options.allow_paths {
                            match trimmed.parse::<PathBuf>() {
                                Ok(path) => {
                                    Self::validate_extension(path.to_str().unwrap(), &options, span.clone(), block);
                                    return;
                                }
                                Err(_) => {}
                            }
                        }

                        ReportsBag::add(report!(
                            kind: ReportKind::Error,
                            message: format!("Value should be a valid URL or path. {err}"),
                            labels: {
                                span.clone() => "Potentially invalid URL or path" => Color::BrightRed
                            },
                            notes: [
                                "Use absolute URLs (https://...) for external resources",
                                "Use relative paths (/path/to/file) for local resources"
                            ]
                        ));
                    }
                }
            }
        })
    }

    pub fn string_prefer_https(self) -> Self {
        self.check_value(|value, span, _| {
            if let Some(s) = value.kind.as_string() {
                let trimmed = s.trim();

                if trimmed.starts_with("http://") {
                    ReportsBag::add(report!(
                        kind: ReportKind::Warning,
                        message: "Consider using HTTPS instead of HTTP".to_string(),
                        labels: {
                            span.clone() => "HTTP URL detected" => Color::BrightYellow
                        },
                        notes: [
                            "HTTPS provides better security for external resources",
                            "Many browsers may block HTTP resources on HTTPS pages"
                        ]
                    ));
                }
            }
        })
    }
}
