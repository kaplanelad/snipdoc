use std::{
    collections::{BTreeMap, HashSet},
    fmt::Write,
    path::{Path, PathBuf},
    str::FromStr,
};

use lazy_static::lazy_static;
use pest::{iterators::Pairs, Parser};
use rayon::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{
    config::InjectConfig,
    db::DBData,
    errors::ParserResult,
    parser::{html_tag, Rule, SnippetKind, SnippetParse, SnippetTemplate},
    read_file::RFile,
    walk::Walk,
    LINE_ENDING,
};

lazy_static! {
    static ref RE_NORMALIZE_TEXT: Regex = Regex::new(r"[\s\r\n]+").unwrap();
    static ref RE_SNIPPET_TEMPLATE_PLACEHOLDER: Regex =
        Regex::new(r"(?m)(^\s*|)\{\s*snippet\}").unwrap();
}

const INJECT_ACTION: &str = "action";
const INJECT_FROM_ATTRIBUTE_NAME: &str = "inject_from";
const STRIP_PREFIX_ATTRIBUTE_NAME: &str = "strip_prefix";
const ADD_PREFIX_ATTRIBUTE_NAME: &str = "add_prefix";
const ADD_TEMPLATE: &str = "template";

/// A struct representing the injection summary result.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct InjectSummary {
    /// Hold all the content in the given input with the snip injection
    /// logic
    pub content: String,
    /// Represent the action that occurred.
    pub actions: Vec<InjectStatus>,
}

#[derive(PartialEq, Eq)]
pub enum InjectAction {
    Copy,
    #[cfg(feature = "exec")]
    Exec,
}

pub struct InjectContentAction {
    pub snippet_id: String,
    pub kind: InjectAction,
    pub inject_from: SnippetKind,
    pub strip_prefix: Option<String>,
    pub add_prefix: Option<String>,
    pub template: Template,
}

#[derive(Default)]
pub enum Template {
    #[default]
    Default,
    Text,
    Json,
    Yaml,
    Toml,
    Html,
    Rust,
    Python,
    Go,
    Sql,
    Shell,
    Bash,
    Sh,
    Custom(String),
}

impl Template {
    #[must_use]
    pub fn new(s: &str) -> Self {
        match s {
            "text" => Self::Text,
            "json" => Self::Json,
            "yaml" => Self::Yaml,
            "toml" => Self::Toml,
            "html" => Self::Html,
            "rust" => Self::Rust,
            "python" => Self::Python,
            "go" => Self::Go,
            "sql" => Self::Sql,
            "shell" => Self::Shell,
            "bash" => Self::Bash,
            "sh" => Self::Sh,
            _ => Self::Custom(s.to_string()),
        }
    }

    #[must_use]
    pub fn before_inject(
        &self,
        content: &str,
        action: &InjectAction,
        custom_templates: &BTreeMap<String, SnippetTemplate>,
    ) -> String {
        match action {
            InjectAction::Copy => {
                let template = match self {
                    Self::Default => content.to_string(),
                    Self::Text => r"```text\n{snippet}\n```".to_string(),
                    Self::Json => r"```json\n{snippet}\n```".to_string(),
                    Self::Yaml => r"```yaml\n{snippet}\n```".to_string(),
                    Self::Toml => r"```toml\n{snippet}\n```".to_string(),
                    Self::Html => r"```html\n{snippet}\n```".to_string(),
                    Self::Rust => r"```rust\n{snippet}\n```".to_string(),
                    Self::Python => r"```python\n{snippet}\n```".to_string(),
                    Self::Go => r"```go\n{snippet}\n```".to_string(),
                    Self::Sql => r"```sql\n{snippet}\n```".to_string(),
                    Self::Shell => r"```shell\n{snippet}\n```".to_string(),
                    Self::Bash => r"```bash\n{snippet}\n```".to_string(),
                    Self::Sh => r"```sh\n{snippet}\n```".to_string(),
                    Self::Custom(template) => custom_templates.get(template).map_or_else(
                        || template.clone(),
                        |custom_template| custom_template.content.clone(),
                    ),
                };

                RE_SNIPPET_TEMPLATE_PLACEHOLDER
                    .replace_all(&template, |caps: &regex::Captures<'_>| {
                        let indent = caps.get(1).map_or("", |m| m.as_str());
                        let snippet_lines = content
                            .lines()
                            .map(|line| format!("{indent}{line}"))
                            .collect::<Vec<_>>()
                            .join(LINE_ENDING);
                        snippet_lines
                    })
                    .replace("\\n", LINE_ENDING)
            }
            #[cfg(feature = "exec")]
            InjectAction::Exec => content.to_string(),
        }
    }

    #[must_use]
    pub fn after_inject(&self, content: &str, action: &InjectAction) -> String {
        match action {
            #[cfg(feature = "exec")]
            InjectAction::Exec => content.to_string(),
            InjectAction::Copy => content.to_string(),
        }
    }
}

impl InjectContentAction {
    pub fn new(attributes: &BTreeMap<String, String>) -> Option<Self> {
        let snippet_id = attributes.get("id").or({
            tracing::debug!(
                attributes = format!("{:?}", attributes),
                "attribute id not found in the given attributes"
            );
            None
        })?;

        let inject_from = attributes.get(INJECT_FROM_ATTRIBUTE_NAME).or({
            tracing::debug!(
                attributes = format!("{:?}", attributes),
                "attribute inject_from not found in the given attributes"
            );
            None
        })?;

        let Ok(inject_from) = SnippetKind::from_str(inject_from) else {
            tracing::debug!(inject_from, "invalid inject_from kind.");
            return None;
        };

        Some(Self {
            snippet_id: snippet_id.to_string(),
            inject_from,
            strip_prefix: attributes.get(STRIP_PREFIX_ATTRIBUTE_NAME).cloned(),
            add_prefix: attributes.get(ADD_PREFIX_ATTRIBUTE_NAME).cloned(),
            template: attributes
                .get(ADD_TEMPLATE)
                .map(|s| Template::new(s))
                .unwrap_or_default(),
            kind: attributes
                .get(INJECT_ACTION)
                .and_then(|a| match a.as_str() {
                    "copy" => Some(InjectAction::Copy),
                    #[cfg(feature = "exec")]
                    "exec" => Some(InjectAction::Exec),
                    _ => None,
                })
                .unwrap_or(InjectAction::Copy),
        })
    }
}

/// The action which occurred
#[derive(Debug, Serialize, Deserialize)]
pub enum InjectStatus {
    /// The snippet found and contains the same content
    Equal { snippet_id: String },
    /// The snippet found and the content was injected
    Injected { snippet_id: String, content: String },
    /// When has injected the snippet but not found snippet
    NotFound {
        snippet_id: String,
        snippet_kind: SnippetKind,
    },
}

pub struct Injector<'a> {
    pub base_folder: &'a Path,
    pub input: &'a str,
    pub config: &'a InjectConfig,
    pub db_data: &'a DBData,
}

/// Represents the inject status result
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Serialize, Deserialize)]
pub struct InjectorResult {
    pub root_folder: PathBuf,
    pub results: InjectSnippets,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InjectSnippets(BTreeMap<PathBuf, InjectedContent>);

/// Represent the injector status result
#[derive(Debug, Serialize, Deserialize)]
pub enum InjectedContent {
    /// When found placeholder snippet section.
    Injected(InjectSummary),
    /// When found a snippet collection but not found inject snippet with the
    /// same it to injector
    None,
    /// When error is encountered
    Error(String),
}

#[derive(Default)]
pub struct InjectStats {
    pub equals: u64,
    pub injects: u64,
    pub inject_unique_files: HashSet<PathBuf>,
    pub errors: BTreeMap<PathBuf, String>,
    pub not_found: BTreeMap<PathBuf, HashSet<String>>,
    pub not_found_count: u64,
}

impl InjectSnippets {
    pub fn iter(&self) -> impl Iterator<Item = (&PathBuf, &InjectedContent)> {
        self.0.iter()
    }

    #[must_use]
    pub fn stats(&self) -> InjectStats {
        let mut stats = InjectStats::default();

        for (file, status) in self.iter() {
            match status {
                InjectedContent::Injected(summary) => {
                    for action in &summary.actions {
                        match action {
                            InjectStatus::Equal { .. } => stats.equals += 1,
                            InjectStatus::Injected { .. } => {
                                stats.injects += 1;
                                stats.inject_unique_files.insert(file.clone());
                            }
                            InjectStatus::NotFound { snippet_id, .. } => {
                                stats
                                    .not_found
                                    .entry(file.clone())
                                    .or_insert_with(|| HashSet::from([snippet_id.to_string()]))
                                    .insert(snippet_id.to_string());
                                stats.not_found_count += 1;
                            }
                        }
                    }
                }
                InjectedContent::None => (),
                InjectedContent::Error(err) => {
                    stats.errors.insert(file.clone(), err.to_string());
                }
            }
        }

        stats
    }
}

impl<'a> Injector<'a> {
    /// Constructs a new [`ParseFile`] with the provided input.
    #[must_use]
    pub const fn new(
        base_folder: &'a Path,
        input: &'a str,
        config: &'a InjectConfig,
        db_data: &'a DBData,
    ) -> Self {
        Self {
            base_folder,
            input,
            config,
            db_data,
        }
    }

    /// Constructs a `Collector` instance by collecting snippets from files
    /// within the provided `Walk`.
    #[must_use]
    pub fn walk(walk: &Walk, db_data: &DBData, config: &InjectConfig) -> InjectorResult {
        let results = walk
            .get_files()
            .par_iter()
            .filter_map(|path| match RFile::new(path) {
                Ok(r_file) => {
                    let status =
                        Self::inject(walk.folder.as_path(), &r_file.content, config, db_data);
                    Some((path.clone(), status))
                }
                Err(_err) => None,
            })
            .collect::<BTreeMap<PathBuf, InjectedContent>>();

        InjectorResult {
            root_folder: walk.folder.clone(),
            results: InjectSnippets(results),
        }
    }

    // Processes a single file and extracts injected snippets.
    ///
    /// # Returns
    ///
    /// Returns `Some` containing the collected snippets if successful,
    /// otherwise returns `None`.
    pub fn inject(
        base_folder: &Path,
        input: &str,
        config: &InjectConfig,
        db_data: &DBData,
    ) -> InjectedContent {
        match Injector::new(base_folder, input, config, db_data).run() {
            Ok(summary) => {
                if summary.actions.is_empty() {
                    tracing::debug!("not found inject content");
                    InjectedContent::None
                } else {
                    tracing::debug!("content injected");
                    InjectedContent::Injected(summary)
                }
            }
            Err(err) => {
                tracing::debug!(err = %err, "could not parse the given content");
                InjectedContent::Error(err.to_string())
            }
        }
    }

    /// Injects snippets in the input file content based on the provided
    /// [`Snippet`] map.
    ///
    /// # Errors
    ///
    /// This function may return an error if it fails to parse the input file.
    /// Other errors encountered during parsing will be logged.
    pub fn run(&self) -> ParserResult<'_, InjectSummary> {
        let pairs = SnippetParse::parse(Rule::file, self.input)?;

        let mut inject_summary = InjectSummary::default();
        self.inject_snippets(pairs, &mut inject_summary)?;

        Ok(inject_summary)
    }

    /// Injects snippets in the input file content based on the provided
    /// `snippets` map.
    ///
    /// # Errors
    ///
    /// This function may return an error if it fails to parse the input file.
    /// Other errors encountered during parsing will be logged.
    ///
    /// # Panics
    ///
    /// This function assumes that the parsing configuration always captures a
    /// snippet containing a tag open. If this assumption is violated, it
    /// indicates a misconfiguration or a critical issue in the parser's
    /// behavior. Consequently, in production code, encountering this panic
    /// indicates a severe problem that requires immediate attention.
    /// In testing scenarios, this panic should be captured to ensure the
    /// correctness of the parser.
    #[allow(clippy::only_used_in_recursion)]
    fn inject_snippets(
        &self,
        pairs: Pairs<'a, Rule>,
        summary: &'a mut InjectSummary,
    ) -> ParserResult<'a, ()> {
        if pairs.len() == 0 {
            return Ok(());
        }

        for pair in pairs {
            let inner = pair.clone().into_inner();

            if pair.as_rule() == Rule::snippet {
                let children: Pairs<'_, Rule> = pair.clone().into_inner();

                let tag_open = html_tag::get_tag_open(&children);
                let tag_close = html_tag::get_tag_close(children.clone());

                let attributes = match html_tag::get_tag_attributes(tag_open) {
                    Ok(attributes) => attributes,
                    Err(err) => {
                        tracing::debug!(tag_open, err = %err, "could not extract attributes from the tag");
                        continue;
                    }
                };

                let inject_content_actions = InjectContentAction::new(&attributes);

                if let Some(inject_actions) = inject_content_actions {
                    if let Some(snippet) = self.db_data.snippets.get(&inject_actions.snippet_id) {
                        if inject_actions.inject_from == SnippetKind::Any
                            || inject_actions.inject_from == snippet.kind
                        {
                            let snippet_content =
                                snippet.create_content(&inject_actions, &self.db_data.templates);

                            let comment_tag = html_tag::get_comment_tag_open(&children);
                            let close_tag_of_tag_open =
                                html_tag::get_comment_tag_of_tag_open(&children);

                            let inject_result = format!(
                                "{comment_tag}{tag_open}{}{snippet_content}{LINE_ENDING}{tag_close}",
                                close_tag_of_tag_open.unwrap_or_default()
                            );

                            summary.content.write_str(&inject_result)?;

                            if Self::is_str_equal(pair.as_str(), &inject_result) {
                                summary.actions.push(InjectStatus::Equal {
                                    snippet_id: inject_actions.snippet_id.to_string(),
                                });
                            } else {
                                summary.actions.push(InjectStatus::Injected {
                                    snippet_id: inject_actions.snippet_id.to_string(),
                                    content: snippet_content,
                                });
                            }
                        } else {
                            // summary.actions.push(InjectStatus::NotFound {
                            //     snippet_id: inject_actions.snippet_id.to_string(),
                            //     snippet_kind: inject_actions.inject_from,
                            // });
                            summary.content.write_str(pair.as_str())?;
                        }
                    } else {
                        summary.actions.push(InjectStatus::NotFound {
                            snippet_id: inject_actions.snippet_id.to_string(),
                            snippet_kind: inject_actions.inject_from,
                        });
                        summary.content.write_str(pair.as_str())?;
                    }
                } else {
                    summary.content.write_str(pair.as_str())?;
                }
            } else {
                self.inject_snippets(inner.clone(), summary)?;
                if inner.len() == 0 {
                    summary.content.write_str(pair.as_str())?;
                }
            }
        }
        Ok(())
    }

    fn is_str_equal(a: &str, b: &str) -> bool {
        RE_NORMALIZE_TEXT.replace_all(a, "") == RE_NORMALIZE_TEXT.replace_all(b, "")
    }
}

#[cfg(not(windows))]
#[cfg(test)]
mod tests {
    use insta::{assert_debug_snapshot, with_settings};

    use super::*;
    use crate::{parser::Snippet, tests_cfg};

    #[test]
    fn get_inject() {
        let content = r#"# Snipdoc

<!-- <snip id="installation" inject_from="code"> -->
# inject `installation` snippet id from code snippet kind
<!-- </snip> -->

<!-- <snip id="inject_from_yaml" inject_from="yaml"> -->
# inject `inject_from_yaml` snippet id from yaml snippet kind
<!-- </snip> -->

<!-- <snip id="inject_from_yaml" inject_from="code"> -->
# Skip injection, `inject_from_yaml` snippet id not exists in code
<!-- </snip> -->

<!-- <snip id="inject_from_yaml" inject_from="any"> -->
# inject_from is any, and this id exists in the yaml
<!-- </snip> -->

<!-- <snip id="description" inject_from="code" add_prefix="//! "> -->
# Adding the prefix for each line
<!-- </snip> -->

<!-- <snip id="description" inject_from="code" strip_prefix="snip"> -->
# Strip `snip` word from prefix for each line
<!-- </snip> -->

<!-- <snip id="description" inject_from="code"
template="```sh\n{snippet}\n```"> --> # Add template to inject snippet
<!-- </snip> -->

<!-- <snip id="description" inject_from="code"> -->
snipdoc
<!-- </snip> -->

<!-- <snip id="not-found" inject_from="code"> -->
not-found
<!-- </snip> -->

"#;

        let snippets: BTreeMap<String, Snippet> = tests_cfg::get_snippet_to_inject();
        let inject_config = InjectConfig::default();
        let base_inject_path = PathBuf::from(".");
        let db_data = DBData {
            snippets,
            templates: BTreeMap::new(),
        };
        let injector = Injector::new(
            base_inject_path.as_path(),
            content,
            &inject_config,
            &db_data,
        );

        with_settings!({filters => tests_cfg::redact::all()}, {
            assert_debug_snapshot!(injector.run());
        });
    }
}
