use crate::{
    Span,
    ast::{
        Attribute, Block, MarstonDocument, Value, ValueKind,
        ident_table::{get_or_intern, resolve},
    },
    html::{lang::is_valid_language_pattern, tags::is_unique_tag},
    info::{BlockInfo, Info},
    report,
    reports::ReportsBag,
};
use ariadne::{Color, Label, Report, ReportKind};
use lasso::Spur;
use rustc_hash::FxHashSet;
use std::{collections::HashSet, fmt::format, path::PathBuf};
use unic_langid::LanguageIdentifier;
use url::Url;

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

#[derive(Clone, Copy, Debug)]
pub enum TargetType {
    Attribute,
    Block,
    Either,
}

pub struct GenericValidator {
    name: Spur,
    target_type: TargetType,
    parent: Option<Vec<Spur>>,
    required: bool,
    type_checks: Vec<Box<dyn Fn(&Value, &Span) -> bool>>,
    value_checks: Vec<Box<dyn Fn(&Value, &Span)>>,
    no_children: bool,
    require_on_of_attrs: Vec<String>,
    disallowed: bool,
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
            no_children: false,
            require_on_of_attrs: Vec::new(),
            disallowed: false,
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

    pub fn in_parent(mut self, parent_names: Vec<&str>) -> Self {
        let parent_keys: Vec<Spur> = parent_names.iter().map(|name| get_or_intern(name)).collect();
        self.parent = Some(parent_keys);
        self
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    pub fn require_one_of_attrs(mut self, attrs: Vec<&str>) -> Self {
        self.require_on_of_attrs = attrs.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn disallowed(mut self) -> Self {
        self.disallowed = true;
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

    pub fn must_be_array(mut self, inner_ty: Option<ValueKind>) -> Self {
        self.type_checks.push(Box::new(move |value: &Value, span: &Span| {
            if let Some(array) = value.kind.as_array() {
                if let Some(inner_ty) = inner_ty.clone() {
                    for item in array {
                        if !item.kind.is_same_kind(&inner_ty) {
                            ReportsBag::add(report!(
                                kind: ReportKind::Error,
                                message: format!("Array item must be of type {:?}", inner_ty),
                                labels: {
                                    span.clone() => "Expected an array of specific type" => Color::BrightRed
                                },
                                notes: ["Ensure all items in the array match the expected type"]
                            ));
                            return false;
                        }
                    }
                }
                true
            } else {
                ReportsBag::add(report!(
                    kind: ReportKind::Error,
                    message: "Value must be an array".to_string(),
                    labels: {
                        span.clone() => "Expected an array value here" => Color::BrightRed
                    },
                    notes: ["Use square brackets to define an array, e.g., [1, 2, 3]"]
                ));
                false
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
            if let Some(s) = value.kind.as_string()
                && s.trim().is_empty()
            {
                ReportsBag::add(report!(
                    kind: ReportKind::Error,
                    message: "String cannot be empty".to_string(),
                    labels: {
                        span.clone() => "This value must not be empty" => Color::BrightRed
                    },
                    notes: ["Provide a meaningful non-empty value"]
                ));
            }
        })
    }

    pub fn disallowed_chars(self, disallowed: Vec<char>) -> Self {
        let disallowed: FxHashSet<char> = disallowed.into_iter().collect();
        self.check_value(move |value, span| {
            if let Some(s) = value.kind.as_string() {
                let chars = s.chars();

                for char in chars {
                    if disallowed.contains(&char) {
                        ReportsBag::add(report!(
                            kind: ReportKind::Error,
                            message: format!("Disallowed character found: '{}'", char),
                            labels: {
                                span.clone() => format!("Disallowed character found: '{}'", char) => Color::BrightRed
                            },
                            notes: ["Remove the disallowed character"]
                        ));
                    }
                }
            }
        })
    }

    pub fn array_not_empty(self) -> Self {
        self.check_value(|value, span| {
            if let Some(array) = value.kind.as_array() {
                if array.is_empty() {
                    ReportsBag::add(report!(
                        kind: ReportKind::Error,
                        message: "Array cannot be empty".to_string(),
                        labels: {
                            span.clone() => "This array must not be empty" => Color::BrightRed
                        },
                    ))
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

                if let Err(_) = trimmed.parse::<LanguageIdentifier>() && !is_valid_language_pattern(&trimmed) {
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
            if let Some(n) = value.kind.as_number() && n < min {
                ReportsBag::add(report!(
                        kind: ReportKind::Error,
                        message: format!("Number must be at least {}", min),
                        labels: {
                            span.clone() => format!("Value {} is below minimum {}", n, min) => Color::BrightRed
                        },
                        notes: [format!("Minimum allowed value is {}", min)]
                    ));
            }
        })
    }

    pub fn number_max(self, max: f64) -> Self {
        self.check_value(move |value, span| {
            if let Some(n) = value.kind.as_number() && n > max {
                ReportsBag::add(report!(
                        kind: ReportKind::Error,
                        message: format!("Number must be at most {}", max),
                        labels: {
                            span.clone() => format!("Value {} exceeds maximum {}", n, max) => Color::BrightRed
                        },
                        notes: [format!("Maximum allowed value is {}", max)]
                    ));
            }
        })
    }

    pub fn number_positive(self) -> Self {
        self.number_min(0.0)
    }

    pub fn validate(&self, doc: &MarstonDocument, info: &mut Info) {
        let name_str = resolve(self.name);

        if let Some(parent_keys) = &self.parent {
            let parents = doc.find_in_parent(parent_keys);

            for parent in parents {
                let found_as_blocks = parent.find_all_child_blocks(self.name);
                let found_as_attrs = parent.get_attribute(self.name);

                self.validate_found_items(
                    found_as_attrs,
                    found_as_blocks,
                    &name_str,
                    &resolve(parent.name().key),
                    parent.name().span,
                )
            }
        } else {
            self.validate_in_document_root(doc, &name_str);
        }
    }

    fn validate_in_document_root(&self, doc: &MarstonDocument, name_str: &str) {
        let mut found_as_attribute = None;
        let mut found_as_block = vec![];

        for block in &doc.blocks {
            if let Some(attr) = block.get_attribute(self.name) {
                found_as_attribute = Some(attr);
            }

            if let Some(block_name) = &block.name
                && block_name.key == self.name
            {
                found_as_block.push(block);
            }

            found_as_block.extend(block.find_all_child_blocks(self.name));
        }

        self.validate_found_items(
            found_as_attribute,
            found_as_block,
            name_str,
            "document root",
            Span::default(),
        );
    }

    pub fn validate_block(&self, block: &Block, name_str: &str) {
        if self.no_children && !block.children.is_empty() {
            ReportsBag::add(report!(
                kind: ReportKind::Error,
                message: format!("'{}' should not have children", name_str),
                labels: {
                    block.name().span => format!("'{}' defined as block but should not have children", name_str) => Color::BrightRed
                },
                notes: [format!("Remove any child blocks or attributes from '{}'", name_str)]
            ));
        }

        let required = self.require_on_of_attrs.clone();
        let mut found = false;
        if required.len() > 0 {
            for attr in &required {
                if let Some(attr) = block.get_attribute(get_or_intern(attr)) {
                    found = true;
                    self.validate_attribute_value(&attr.value, &attr.value.span);
                }
            }
            if !found {
                ReportsBag::add(report!(
                    kind: ReportKind::Error,
                    message: format!("'{}' should have one of the following attributes: {:?}", name_str, required),
                    labels: {
                        block.name().span => format!("'{}' should have at least one of the following attributes: {:?}", name_str, required) => Color::BrightRed
                    },
                    notes: [format!("Add one of the following attributes to '{}': {:?}", name_str, required)]
                ))
            }
        }
    }

    fn validate_found_items(
        &self,
        found_as_attribute: Option<&Attribute>,
        found_as_block: Vec<&Block>,
        name_str: &str,
        parent: &str,
        parent_span: Span,
    ) {
        let blocks = if found_as_block.is_empty() { None } else { Some(found_as_block) };

        match (found_as_attribute, blocks, self.target_type) {
            (Some(attr), None, TargetType::Attribute) | (Some(attr), None, TargetType::Either) => {
                self.validate_attribute_value(&attr.value, &attr.value.span);
            }
            (None, Some(blocks), TargetType::Block) | (None, Some(blocks), TargetType::Either) => {
                for block in blocks {
                    if self.disallowed {
                        ReportsBag::add(report!(
                            kind: ReportKind::Error,
                            message: format!("'{}' should not be used", name_str),
                            labels: {
                                block.name().span.clone() => format!("'{}' found as block but should not be used", name_str) => Color::BrightRed
                            },
                            notes: [format!("Remove '{}' from '{}'", name_str, parent)]
                        ));
                    } else {
                        self.validate_block(block, name_str);
                    }
                }
            }
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
            (_, Some(blocks), TargetType::Attribute) => {
                for block in blocks {
                    let block_span = if let Some(name) = &block.name {
                        name.span.clone()
                    } else {
                        Span::default()
                    };
                    ReportsBag::add(report!(
                        kind: ReportKind::Error,
                        message: format!("'{}' should be an attribute, not a block", name_str),
                        labels: {
                            block_span => format!("'{}' found as block but expected as attribute", name_str) => Color::BrightRed
                        },
                        notes: [format!("Define '{}' as an attribute instead of a block", name_str)]
                    ));
                }
            }
            (Some(attr), Some(blocks), _) => {
                for block in blocks {
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
                            parent_span => format!("'{}' {} is required in '{}'", name_str, expected_type, parent) => Color::BrightRed
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

    pub fn string_valid_url(self, options: Option<ValidUrlOptions>) -> Self {
        let options = options.unwrap_or_default();

        self.check_value(move |value, span| {
            if let Some(s) = value.kind.as_string() {
                let trimmed = s.trim();

                match Url::parse(trimmed) {
                    Ok(url) => {
                        if options.disallowed_protocols.iter().any(|&p| p == url.scheme()) {
                            ReportsBag::add(report!(
                                kind: ReportKind::Error,
                                message: format!("Found disallowed URL protocol: {}", url.scheme()),
                                labels: {
                                    span.clone() => "disallowed invalid URL" => Color::BrightRed
                                },
                            ));
                        }
                    }
                    Err(err) => {
                        if options.allow_paths && trimmed.parse::<PathBuf>().is_ok() {
                            return;
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

    pub fn string_file_extension(self, extension: &'static str) -> Self {
        self.check_value(move |value, span| {
            if let Some(s) = value.kind.as_string() {
                let trimmed = s.trim();
                let expected_ext = if extension.starts_with('.') { extension } else { &format!(".{}", extension) };

                if !trimmed.ends_with(expected_ext) {
                    ReportsBag::add(report!(
                        kind: ReportKind::Warning,
                        message: format!("File should have {} extension", expected_ext),
                        labels: {
                            span.clone() => format!("Expected {} file", expected_ext) => Color::BrightYellow
                        },
                        notes: [format!("Ensure the file has a {} extension", expected_ext)]
                    ));
                }
            }
        })
    }

    pub fn string_prefer_https(self) -> Self {
        self.check_value(|value, span| {
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

    pub fn string_allowed_values(self, allowed: &'static [&'static str]) -> Self {
        self.check_value(move |value, span| {
            if let Some(s) = value.kind.as_string() {
                let trimmed = s.trim().to_lowercase();

                if !allowed.iter().any(|&v| trimmed == v.to_lowercase()) {
                    ReportsBag::add(report!(
                        kind: ReportKind::Warning,
                        message: format!("Uncommon value '{}' specified", s.trim()),
                        labels: {
                            span.clone() => "Consider if this value is appropriate" => Color::BrightYellow
                        },
                        notes: [
                            format!("Common values are: {}", allowed.join(", ")),
                            "Other values may not be supported by all browsers"
                        ]
                    ));
                }
            }
        })
    }

    pub fn block_no_children(mut self) -> Self {
        self.no_children = true;
        self
    }
}

#[derive(Default)]
pub struct ValidUrlOptions {
    pub disallowed_protocols: &'static [&'static str],
    pub allow_paths: bool,
}

impl ValidUrlOptions {
    pub fn new(disallowed_protocols: &'static [&'static str], allow_paths: bool) -> Self {
        Self { disallowed_protocols, allow_paths }
    }
}

pub fn validate_block_no_children(block: &Block, block_type: &str) {
    if !block.children.is_empty() {
        ReportsBag::add(report!(
            kind: ReportKind::Error,
            message: format!("{} blocks should not contain any children", block_type),
            labels: {
                block.span.clone() => format!("{} block contains children", block_type) => Color::BrightRed
            },
            notes: [
                format!("{} blocks should only contain attributes", block_type),
                "Move any children to separate blocks or external files"
            ]
        ));
    }
}

pub fn validate_mutually_exclusive_attributes(
    block: &Block,
    attr1_name: &str,
    attr2_name: &str,
    warning_message: &str,
    notes: &[&str],
) {
    let attr1_key = get_or_intern(attr1_name);
    let attr2_key = get_or_intern(attr2_name);

    let has_attr1 = block.get_attribute(attr1_key).is_some();
    let has_attr2 = block.get_attribute(attr2_key).is_some();

    if has_attr1 && has_attr2 {
        let mut report = Report::build(ReportKind::Warning, (ReportsBag::file(), Span::default()))
            .with_message(warning_message.to_string())
            .with_label(
                Label::new((ReportsBag::file(), block.span.clone()))
                    .with_message(format!(
                        "Conflicting '{}' and '{}' attributes",
                        attr1_name, attr2_name
                    ))
                    .with_color(Color::BrightYellow),
            );

        for note in notes.iter() {
            report = report.with_note(note.to_string());
        }

        ReportsBag::add(report.finish());
    }
}

pub fn validate_block_not_empty(block: &Block, block_type: &str, src_attr_name: Option<&str>) {
    let has_src = if let Some(attr_name) = src_attr_name {
        block.get_attribute(get_or_intern(attr_name)).is_some()
    } else {
        false
    };

    let has_content = !block.children.is_empty();

    if !has_src && !has_content {
        let suggestion = if let Some(attr_name) = src_attr_name {
            format!("Either provide a '{}' attribute or add content", attr_name)
        } else {
            "Add content to this block".to_string()
        };

        ReportsBag::add(report!(
            kind: ReportKind::Warning,
            message: format!("{} block appears to be empty", block_type),
            labels: {
                block.span.clone() => format!("Empty {} block", block_type) => Color::BrightYellow
            },
            notes: [
                suggestion,
                format!("Or consider removing this empty {} block", block_type)
            ]
        ));
    }
}
