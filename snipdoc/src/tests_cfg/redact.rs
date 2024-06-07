#[cfg(windows)]
pub const REGEX_REPLACE_LINE_ENDING: &str = r"\\r\\n";

#[cfg(not(windows))]
pub const REGEX_REPLACE_LINE_ENDING: &str = r"\\n";

#[must_use]
pub fn all() -> Vec<(&'static str, &'static str)> {
    vec![(REGEX_REPLACE_LINE_ENDING, "[NEW_LINE]")]
}
