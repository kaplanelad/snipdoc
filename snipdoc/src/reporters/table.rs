use std::{collections::BTreeMap, path::Path};

use tabled::{builder::Builder, settings::Style};

use super::ReporterOutput;
use crate::{
    db::Snippet,
    parser::injector::InjectAction,
    processor::{InjectContentResult, InjectResults},
};

pub struct Output {}

impl Output {}

impl ReporterOutput for Output {
    fn snippets(&self, root_folder: &Path, snippets: &BTreeMap<String, Snippet>) {
        let mut builder = Builder::default();
        builder.push_record(["#", "Snippet Kind", "ID", "Path"]);

        println!("Found {} snippets", snippets.len());
        let mut count = 1;
        for (id, snippet) in snippets {
            let path_view = std::fs::canonicalize(root_folder)
                .map(|absolute_path| {
                    snippet
                        .path
                        .strip_prefix(absolute_path)
                        .unwrap_or(&snippet.path)
                })
                .unwrap_or(&snippet.path);

            builder.push_record([
                format!("{count}"),
                format!("{:#?}", snippet.kind),
                id.to_string(),
                format!("{}", path_view.display()),
            ]);
            count += 1;
        }

        println!("{}", builder.build().with(Style::modern()));
    }

    fn inject(&self, root_folder: &Path, result: &InjectResults) {
        let mut builder = Builder::default();
        builder.push_record(["Path", "Action", "Snippet ID", ""]);

        for (file, status) in result.iter() {
            let path_view = std::fs::canonicalize(root_folder)
                .map(|absolute_path| file.strip_prefix(absolute_path).unwrap_or(file))
                .unwrap_or(file);

            match status {
                InjectContentResult::Injected(summary) => {
                    for action in &summary.actions {
                        match action {
                            InjectAction::Equal { snippet_id } => {
                                builder.push_record([
                                    format!("{}", path_view.display()),
                                    "equal".to_string(),
                                    snippet_id.to_string(),
                                    String::new(),
                                ]);
                            }
                            InjectAction::Injected {
                                snippet_id,
                                content: _,
                            } => {
                                builder.push_record([
                                    format!("{}", path_view.display()),
                                    "injected".to_string(),
                                    snippet_id.to_string(),
                                    String::new(),
                                ]);
                            }
                            InjectAction::NotFound {
                                snippet_id,
                                snippet_kind,
                            } => {
                                builder.push_record([
                                    format!("{}", path_view.display()),
                                    "not-found-snippets".to_string(),
                                    snippet_id.to_string(),
                                    format!("Inject from: {snippet_kind:?}"),
                                ]);
                            }
                        }
                    }
                }
                InjectContentResult::None => (),
                InjectContentResult::Error(error) => {
                    builder.push_record([
                        format!("{}", path_view.display()),
                        "error".to_string(),
                        String::new(),
                        error.to_string(),
                    ]);
                }
            }
        }

        println!("{}", builder.build().with(Style::modern()));
    }
}
