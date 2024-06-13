#[cfg(windows)]
pub const REGEX_REPLACE_LINE_ENDING: &str = r"\\r\\n";

#[cfg(not(windows))]
pub const REGEX_REPLACE_LINE_ENDING: &str = r"\\n";

pub const REDACT_NEW_LINE: &str = "[NEW_LINE]";
#[must_use]
pub fn all() -> Vec<(&'static str, &'static str)> {
    vec![(REGEX_REPLACE_LINE_ENDING, REDACT_NEW_LINE)]
}
