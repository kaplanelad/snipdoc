use std::{
    collections::{BTreeMap, HashSet},
    path::{Path, PathBuf},
};

use console::style;

use super::ReporterOutput;
use crate::{db::Snippet, parser::injector::InjectAction, processor::InjectResult};

pub struct Output {}

impl Output {}

#[derive(Default)]
struct CalculateData<'a> {
    actions: usize,
    errors: BTreeMap<&'a PathBuf, &'a String>,
    inject_count: usize,
    inject_files: HashSet<&'a PathBuf>,
    not_found: BTreeMap<&'a PathBuf, HashSet<&'a String>>,
    not_found_count: usize,
    equal: usize,
}

impl ReporterOutput for Output {
    fn snippets(&self, _: &Path, snippets: &BTreeMap<String, Snippet>) {
        println!(
            "{}",
            style(format!("Found {} snippets", snippets.len()))
                .green()
                .bold()
        );
        let mut count = 1;
        for (id, snippet) in snippets {
            println!(
                "{:<5} {:<10} {:<40} {}",
                count,
                format!("{:?}", snippet.kind),
                id,
                snippet.path.display()
            );

            count += 1;
        }
    }

    #[allow(clippy::too_many_lines)]
    fn inject(&self, root_folder: &Path, result: &BTreeMap<PathBuf, InjectResult>) {
        let mut data = CalculateData::default();

        for (file, status) in result {
            match status {
                InjectResult::Injected(summary) => {
                    for action in &summary.actions {
                        match action {
                            InjectAction::Equal { snippet_id: _ } => data.equal += 1,
                            InjectAction::Injected {
                                snippet_id: _,
                                content: _,
                            } => {
                                data.inject_files.insert(file);
                                data.inject_count += 1;
                                data.actions += 1;
                            }
                            InjectAction::NotFound {
                                snippet_id,
                                snippet_kind: _,
                            } => {
                                // data.not_found.insert(snippet_id);
                                data.not_found
                                    .entry(file)
                                    .or_insert_with(|| HashSet::from([snippet_id]))
                                    .insert(snippet_id);
                                data.not_found_count += 1;
                            }
                        }
                    }
                }
                InjectResult::None => (),
                InjectResult::Error(err) => {
                    data.errors.insert(file, err);
                }
            };
        }

        println!("==============================");
        println!("{}", style("       Snipdoc ").green().bold());
        println!("==============================");
        println!();
        println!("{}", style("Overall Summary:").bold());
        println!(
            "Folder                : {}",
            std::fs::canonicalize(root_folder)
                .unwrap_or_else(|_| root_folder.to_path_buf())
                .display()
        );
        println!();
        println!("{}", style("Detailed Summary by Action Type:").bold());
        println!("{}", style(format!("Equal      : {}", data.equal)).cyan());
        println!(
            "{}",
            style(format!("Injected   : {}", style(data.inject_count))).green()
        );

        if !data.not_found.is_empty() {
            println!(
                "{}",
                style(format!("Not Found  : {}", data.not_found_count)).yellow()
            );
        }
        if !data.errors.is_empty() {
            println!(
                "{}",
                style(format!("Error      : {}", data.errors.len())).red()
            );
        }

        if !data.not_found.is_empty() {
            println!();
            println!("{}", style("Not Found:").bold());

            let mut entries: Vec<_> = data.not_found.iter().collect();
            entries.sort_by(|(file1, _), (file2, _)| file1.cmp(file2));

            for (file, snippet_ids) in entries {
                let path_view = std::fs::canonicalize(root_folder)
                    .map(|absolute_path| file.strip_prefix(absolute_path).unwrap_or(file))
                    .unwrap_or(file);

                let mut sorted_snippet_ids: Vec<_> = snippet_ids.iter().copied().collect();
                sorted_snippet_ids.sort();

                for snippet_id in sorted_snippet_ids {
                    println!(" - {}, snippet id: {}", path_view.display(), snippet_id);
                }
            }
        }

        if !data.errors.is_empty() {
            println!();
            println!("{}", style("Errors:").bold());
            for (file, error_msg) in data.errors {
                let path_view = std::fs::canonicalize(root_folder)
                    .map(|absolute_path| file.strip_prefix(absolute_path).unwrap_or(file))
                    .unwrap_or(file);

                println!(" - {} : {error_msg}", path_view.display());
            }
        }

        if !data.inject_files.is_empty() {
            println!();
            println!("Injected In Files:");
            for file in data.inject_files {
                let path_view = std::fs::canonicalize(root_folder)
                    .map(|absolute_path| file.strip_prefix(absolute_path).unwrap_or(file))
                    .unwrap_or(file);
                println!(" - {}", path_view.display());
            }
        }
    }
}
