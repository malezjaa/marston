use once_cell::sync::Lazy;
use regex::Regex;

pub fn is_valid_language_pattern(code: &str) -> bool {
    static RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"^[a-zA-Z]{2,3}(-[a-zA-Z0-9]{2,3})?$").unwrap());

    RE.is_match(code)
}
