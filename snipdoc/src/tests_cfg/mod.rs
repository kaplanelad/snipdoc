use std::path::PathBuf;

use crate::{
    db::{Snippet, SnippetKind},
    parser::collector::CollectSnippet,
};

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
        content: "$ cargo install snipdoc\n$ snipdoc --version".to_string(),
        kind: SnippetKind::Code,
        path: PathBuf::from("main.rs"),
    }
}
