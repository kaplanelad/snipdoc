use std::{collections::HashMap, path::PathBuf};

use crate::parser::{collector::CollectSnippet, Snippet, SnippetKind};

#[must_use]
pub fn get_collect_snippets() -> Vec<CollectSnippet> {
    vec![
        CollectSnippet {
            id: "description".to_string(),
            snippet: vec!["test".to_string(), "snipdoc".to_string()],
            inject_from: None,
            tag_open: "<snip id=\"description\">".to_string(),
            tag_close: "<!-- </snip> -->\n".to_string(),
        },
        CollectSnippet {
            id: "installation".to_string(),
            snippet: vec![
                "```".to_string(),
                "cargo install snipdoc".to_string(),
                "```".to_string(),
            ],
            inject_from: None,
            tag_open: "<snip id=\"install\">".to_string(),
            tag_close: "<!-- </snip> -->\n".to_string(),
        },
        CollectSnippet {
            id: "from-yaml".to_string(),
            snippet: vec!["ignore-snippet".to_string()],
            inject_from: Some(SnippetKind::Yaml),
            tag_open: "<snip id=\"from-yaml\">".to_string(),
            tag_close: "<!-- </snip> -->\n".to_string(),
        },
    ]
}

#[must_use]
pub fn get_snippet() -> Snippet {
    Snippet {
        id: "test".to_string(),
        content: "$ cargo install snipdoc\n$ snipdoc --version".to_string(),
        kind: SnippetKind::Code,
        path: PathBuf::from("main.rs"),
    }
}

#[must_use]
pub fn get_snippet_to_inject() -> HashMap<String, Snippet> {
    HashMap::from([
        (
            "description".to_string(),
            Snippet {
                id: "test".to_string(),
                content: "snipdoc".to_string(),
                kind: SnippetKind::Code,
                path: PathBuf::from("main.rs"),
            },
        ),
        (
            "installation".to_string(),
            Snippet {
                id: "test".to_string(),
                content: "$ cargo install snipdoc\n$ snipdoc --version".to_string(),
                kind: SnippetKind::Code,
                path: PathBuf::from("main.rs"),
            },
        ),
        (
            "inject_from_yaml".to_string(),
            Snippet {
                id: "test".to_string(),
                content: "inject_from_yaml".to_string(),
                kind: SnippetKind::Yaml,
                path: PathBuf::from("main.rs"),
            },
        ),
    ])
}
