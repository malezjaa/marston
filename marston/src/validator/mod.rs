use crate::{
    Span,
    ast::{
        Attribute, Block, MarstonDocument, Value,
        ident_table::{get_or_intern, resolve},
    },
    html::{lang::is_valid_language_pattern, tags::is_unique_tag},
    info::{BlockInfo, Info},
    report,
    reports::ReportsBag,
};
use ariadne::{Color, Label, Report, ReportKind};
use lasso::Spur;

pub mod rules;

pub type ValidationRule<T> = fn(&T, &mut Info);

pub trait Validate: Sized {
    fn rules() -> Vec<ValidationRule<Self>>;

    fn call_rules(&self, info: &mut Info) {
        for rule in Self::rules() {
            rule(self, info);
        }
    }

    fn validate(&self, info: &mut Info);
}

#[derive(Clone, Copy)]
pub enum TargetType {
    Attribute,
    Block,
    Either,
}

pub struct GenericValidator {
    name: Spur,
    target_type: TargetType,
    parent: Option<Spur>,
    required: bool,
    type_checks: Vec<Box<dyn Fn(&Value, &Span) -> bool>>,
    value_checks: Vec<Box<dyn Fn(&Value, &Span)>>,
}

impl GenericValidator {
    pub fn new(name: &str) -> Self {
        Self {
            name: get_or_intern(name),
            target_type: TargetType::Either,
            parent: None,
            required: false,
            type_checks: Vec::new(),
            value_checks: Vec::new(),
        }
    }

    pub fn as_attribute(mut self) -> Self {
        self.target_type = TargetType::Attribute;
        self
    }

    pub fn as_block(mut self) -> Self {
        self.target_type = TargetType::Block;
        self
    }

    pub fn in_parent(mut self, parent_name: &str) -> Self {
        self.parent = Some(get_or_intern(parent_name));
        self
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    pub fn must_be_string(mut self) -> Self {
        self.type_checks.push(Box::new(|value: &Value, span: &Span| {
            if value.kind.as_string().is_none() {
                ReportsBag::add(report!(
                    kind: ReportKind::Error,
                    message: "Value must be a string".to_string(),
                    labels: {
                        span.clone() => "Expected a string value here" => Color::BrightRed
                    },
                    notes: ["Use quotes to define a string value, e.g., \"my value\""]
                ));
                false
            } else {
                true
            }
        }));
        self
    }

    pub fn must_be_number(mut self) -> Self {
        self.type_checks.push(Box::new(|value: &Value, span: &Span| {
            if value.kind.as_number().is_none() {
                ReportsBag::add(report!(
                    kind: ReportKind::Error,
                    message: "Value must be a number".to_string(),
                    labels: {
                        span.clone() => "Expected a numeric value here" => Color::BrightRed
                    },
                    notes: ["Use a numeric value, e.g., 42 or 3.14"]
                ));
                false
            } else {
                true
            }
        }));
        self
    }

    pub fn must_be_boolean(mut self) -> Self {
        self.type_checks.push(Box::new(|value: &Value, span: &Span| {
            if value.kind.as_boolean().is_none() {
                ReportsBag::add(report!(
                    kind: ReportKind::Error,
                    message: "Value must be a boolean".to_string(),
                    labels: {
                        span.clone() => "Expected true or false here" => Color::BrightRed
                    },
                    notes: ["Use 'true' or 'false' as the value"]
                ));
                false
            } else {
                true
            }
        }));
        self
    }

    pub fn check_value<F>(mut self, check: F) -> Self
    where
        F: Fn(&Value, &Span) + 'static,
    {
        self.value_checks.push(Box::new(check));
        self
    }

    pub fn string_not_empty(self) -> Self {
        self.check_value(|value, span| {
            if let Some(s) = value.kind.as_string() {
                if s.trim().is_empty() {
                    ReportsBag::add(report!(
                        kind: ReportKind::Error,
                        message: "String cannot be empty".to_string(),
                        labels: {
                            span.clone() => "This value must not be empty" => Color::BrightRed
                        },
                        notes: ["Provide a meaningful non-empty value"]
                    ));
                }
            }
        })
    }

    pub fn string_min_length(self, min: usize) -> Self {
        self.check_value(move |value, span| {
            if let Some(s) = value.kind.as_string() {
                let trimmed = s.trim();
                if trimmed.len() < min {
                    ReportsBag::add(report!(
                        kind: ReportKind::Warning,
                        message: format!("String too short (minimum {} characters)", min),
                        labels: {
                            span.clone() => "Consider making this more descriptive" => Color::BrightYellow
                        },
                        notes: [format!("Minimum recommended length is {} characters", min)]
                    ));
                }
            }
        })
    }

    pub fn string_min_length_error(self, min: usize) -> Self {
        self.check_value(move |value, span| {
            if let Some(s) = value.kind.as_string() {
                let trimmed = s.trim();
                if trimmed.len() < min {
                    ReportsBag::add(report!(
                        kind: ReportKind::Error,
                        message: format!("String too short (minimum {} characters)", min),
                        labels: {
                            span.clone() => "This value is too short" => Color::BrightRed
                        },
                        notes: [format!("Minimum required length is {} characters", min)]
                    ));
                }
            }
        })
    }

    pub fn string_max_length(self, max: usize) -> Self {
        self.check_value(move |value, span| {
            if let Some(s) = value.kind.as_string() {
                let trimmed = s.trim();
                if trimmed.len() > max {
                    ReportsBag::add(report!(
                        kind: ReportKind::Warning,
                        message: format!("String exceeds {} characters", max),
                        labels: {
                            span.clone() => "Consider shortening this value" => Color::BrightYellow
                        },
                        notes: [format!("Maximum recommended length is {} characters", max)]
                    ));
                }
            }
        })
    }

    pub fn string_max_length_error(self, max: usize) -> Self {
        self.check_value(move |value, span| {
            if let Some(s) = value.kind.as_string() {
                let trimmed = s.trim();
                if trimmed.len() > max {
                    ReportsBag::add(report!(
                        kind: ReportKind::Error,
                        message: format!("String exceeds {} characters", max),
                        labels: {
                            span.clone() => "This value is too long" => Color::BrightRed
                        },
                        notes: [format!("Maximum allowed length is {} characters", max)]
                    ));
                }
            }
        })
    }

    pub fn string_not_generic(self, generic_values: &'static [&'static str]) -> Self {
        self.check_value(move |value, span| {
            if let Some(s) = value.kind.as_string() {
                let lower = s.trim().to_lowercase();
                if generic_values.iter().any(|&v| lower == v) {
                    ReportsBag::add(report!(
                        kind: ReportKind::Warning,
                        message: "Generic or uninformative value".to_string(),
                        labels: {
                            span.clone() => "Try using a more specific value" => Color::BrightYellow
                        },
                        notes: ["Generic values provide little context or value"]
                    ));
                }
            }
        })
    }

    pub fn string_valid_language_code(self) -> Self {
        self.check_value(|value, span| {
            if let Some(s) = value.kind.as_string() {
                let trimmed = s.trim().to_lowercase();
                let valid_codes = [
                    "en", "es", "fr", "de", "it", "pt", "ru", "ja", "ko", "zh", "ar", "hi",
                    "en-us", "en-gb", "es-es", "es-mx", "fr-fr", "fr-ca", "de-de", "it-it",
                    "pt-br", "pt-pt", "zh-cn", "zh-tw", "ja-jp", "ko-kr"
                ];

                if !valid_codes.contains(&trimmed.as_str()) && !is_valid_language_pattern(&trimmed) {
                    ReportsBag::add(report!(
                        kind: ReportKind::Warning,
                        message: "Potentially invalid language code".to_string(),
                        labels: {
                            span.clone() => "This doesn't appear to be a standard language code" => Color::BrightYellow
                        },
                        notes: [
                            "Use ISO 639-1 language codes (e.g., 'en', 'es', 'fr')",
                            "Or include region codes (e.g., 'en-US', 'es-MX')"
                        ]
                    ));
                }
            }
        })
    }

    pub fn number_min(self, min: f64) -> Self {
        self.check_value(move |value, span| {
            if let Some(n) = value.kind.as_number() {
                if n < min {
                    ReportsBag::add(report!(
                        kind: ReportKind::Error,
                        message: format!("Number must be at least {}", min),
                        labels: {
                            span.clone() => format!("Value {} is below minimum {}", n, min) => Color::BrightRed
                        },
                        notes: [format!("Minimum allowed value is {}", min)]
                    ));
                }
            }
        })
    }

    pub fn number_max(self, max: f64) -> Self {
        self.check_value(move |value, span| {
            if let Some(n) = value.kind.as_number() {
                if n > max {
                    ReportsBag::add(report!(
                        kind: ReportKind::Error,
                        message: format!("Number must be at most {}", max),
                        labels: {
                            span.clone() => format!("Value {} exceeds maximum {}", n, max) => Color::BrightRed
                        },
                        notes: [format!("Maximum allowed value is {}", max)]
                    ));
                }
            }
        })
    }

    pub fn number_positive(self) -> Self {
        self.number_min(0.0)
    }

    pub fn validate(&self, doc: &MarstonDocument, info: &mut Info) {
        let name_str = resolve(self.name);

        let parent_block = if let Some(parent_key) = self.parent {
            match doc.find_block_by_name(parent_key) {
                Some(block) => block,
                None => {
                    if self.required {
                        let parent_name = resolve(parent_key);
                        ReportsBag::add(report!(
                            kind: ReportKind::Error,
                            message: format!("Parent block '{}' not found", parent_name),
                            labels: {
                                Span::default() => format!("Cannot validate '{}' without parent '{}'", name_str, parent_name) => Color::BrightRed
                            },
                            notes: [format!("Ensure the '{}' block exists", parent_name)]
                        ));
                    }
                    return;
                }
            }
        } else {
            return self.validate_in_document_root(doc, &name_str);
        };

        let found_as_attribute = parent_block.get_attribute(self.name);
        let found_as_block = parent_block.find_child_block(self.name);

        self.validate_found_items(
            found_as_attribute,
            found_as_block,
            &name_str,
            &resolve(parent_block.name.as_ref().map(|n| n.key).unwrap_or(Spur::default())),
            parent_block.span.clone(),
        );
    }

    fn validate_in_document_root(&self, doc: &MarstonDocument, name_str: &str) {
        let mut found_as_attribute = None;
        let mut found_as_block = None;

        for block in &doc.blocks {
            if let Some(attr) = block.get_attribute(self.name) {
                found_as_attribute = Some(attr);
            }

            if let Some(block_name) = &block.name {
                if block_name.key == self.name {
                    found_as_block = Some(block);
                }
            }

            if let Some(child_block) = block.find_child_block(self.name) {
                found_as_block = Some(child_block);
            }
        }

        self.validate_found_items(
            found_as_attribute,
            found_as_block,
            name_str,
            "document root",
            Span::default(),
        );
    }

    fn validate_found_items(
        &self,
        found_as_attribute: Option<&Attribute>,
        found_as_block: Option<&Block>,
        name_str: &str,
        parent_name: &str,
        parent_span: Span,
    ) {
        match (found_as_attribute, found_as_block, self.target_type) {
            (Some(attr), None, TargetType::Attribute) | (Some(attr), None, TargetType::Either) => {
                self.validate_attribute_value(&attr.value, &attr.value.span);
            }
            (None, Some(_block), TargetType::Block) | (None, Some(_block), TargetType::Either) => {}
            (Some(attr), _, TargetType::Block) => {
                ReportsBag::add(report!(
                    kind: ReportKind::Error,
                    message: format!("'{}' should be a block, not an attribute", name_str),
                    labels: {
                        attr.value.span.clone() => format!("'{}' found as attribute but expected as block", name_str) => Color::BrightRed
                    },
                    notes: [format!("Define '{}' as a block instead of an attribute", name_str)]
                ));
            }
            (_, Some(block), TargetType::Attribute) => {
                let block_span =
                    if let Some(name) = &block.name { name.span.clone() } else { Span::default() };
                ReportsBag::add(report!(
                    kind: ReportKind::Error,
                    message: format!("'{}' should be an attribute, not a block", name_str),
                    labels: {
                        block_span => format!("'{}' found as block but expected as attribute", name_str) => Color::BrightRed
                    },
                    notes: [format!("Define '{}' as an attribute instead of a block", name_str)]
                ));
            }
            (Some(attr), Some(block), _) => {
                ReportsBag::add(report!(
                    kind: ReportKind::Error,
                    message: format!("'{}' defined as both attribute and block", name_str),
                    labels: {
                        attr.value.span.clone() => format!("'{}' defined as attribute here", name_str) => Color::BrightRed,
                        block.name.as_ref().map(|n| n.span.clone()).unwrap_or_default() => format!("'{}' defined as block here", name_str) => Color::BrightRed
                    },
                    notes: [format!("'{}' should be defined only once, either as attribute or block", name_str)]
                ));
            }
            (None, None, _) => {
                if self.required {
                    let expected_type = match self.target_type {
                        TargetType::Attribute => "attribute",
                        TargetType::Block => "block",
                        TargetType::Either => "attribute or block",
                    };
                    ReportsBag::add(report!(
                        kind: ReportKind::Error,
                        message: format!("Missing required {} '{}'", expected_type, name_str),
                        labels: {
                            parent_span => format!("'{}' {} is required in '{}'", name_str, expected_type, parent_name) => Color::BrightRed
                        },
                        notes: [format!("Add the required '{}' {}", name_str, expected_type)]
                    ));
                }
            }
        }
    }

    fn validate_attribute_value(&self, value: &Value, span: &Span) {
        for type_check in &self.type_checks {
            if !type_check(value, span) {
                return;
            }
        }

        for value_check in &self.value_checks {
            value_check(value, span);
        }
    }
}
