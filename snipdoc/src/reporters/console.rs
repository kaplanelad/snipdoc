use std::{
    collections::{BTreeMap, HashSet},
    path::{Path, PathBuf},
};

use console::style;

use super::ReporterOutput;
use crate::parser::{
    injector::{InjectSnippets, InjectStats},
    Snippet,
};

pub struct Output {}

impl Output {}

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

    fn inject(&self, root_folder: &Path, result: &InjectSnippets) {
        let stats = result.stats();

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
        println!("{}", style(format!("Equal      : {}", stats.equals)).cyan());
        println!(
            "{}",
            style(format!("Injected   : {}", style(stats.injects))).green()
        );

        if !stats.not_found.is_empty() {
            println!(
                "{}",
                style(format!("Not Found  : {}", stats.not_found_count)).yellow()
            );
        }
        if !stats.errors.is_empty() {
            println!(
                "{}",
                style(format!("Error      : {}", stats.errors.len())).red()
            );
        }

        if !stats.not_found.is_empty() {
            Self::print_not_found_snippets_to_inject(root_folder, &stats.not_found);
        }

        if !stats.errors.is_empty() {
            Self::print_errors(root_folder, &stats.errors);
        }

        if !stats.inject_unique_files.is_empty() {
            Self::print_inject_files(
                root_folder,
                "Injected In Files:",
                &stats.inject_unique_files,
            );
        }
    }

    fn check(&self, root_folder: &Path, stats: &InjectStats) {
        if !stats.inject_unique_files.is_empty() {
            Self::print_inject_files(
                root_folder,
                "Snippets should updated in:",
                &stats.inject_unique_files,
            );
        }

        if !stats.errors.is_empty() {
            Self::print_errors(root_folder, &stats.errors);
        }

        if !stats.not_found.is_empty() {
            Self::print_not_found_snippets_to_inject(root_folder, &stats.not_found);
        }
    }
}

impl Output {
    fn print_errors(root_folder: &Path, errors: &BTreeMap<PathBuf, String>) {
        println!();
        println!("{}", style("Found errors in the following files:").bold());
        for (file, error_msg) in errors {
            let path_view = std::fs::canonicalize(root_folder)
                .map(|absolute_path| file.strip_prefix(absolute_path).unwrap_or(file))
                .unwrap_or(file);

            println!(" - {} : {error_msg}", path_view.display());
        }
    }

    fn print_inject_files(root_folder: &Path, title: &str, inject_files: &HashSet<PathBuf>) {
        println!();
        println!("{title}");
        for file in inject_files {
            let path_view = std::fs::canonicalize(root_folder)
                .map(|absolute_path| file.strip_prefix(absolute_path).unwrap_or(file))
                .unwrap_or(file);
            println!(" - {}", path_view.display());
        }
    }

    fn print_not_found_snippets_to_inject(
        root_folder: &Path,
        not_found: &BTreeMap<PathBuf, HashSet<String>>,
    ) {
        println!();
        println!("{}", style("Snippets to inject not found:").bold());

        let mut entries: Vec<_> = not_found.iter().collect();
        entries.sort_by(|(file1, _), (file2, _)| file1.cmp(file2));

        for (file, snippet_ids) in entries {
            let path_view = std::fs::canonicalize(root_folder)
                .map(|absolute_path| file.strip_prefix(absolute_path).unwrap_or(file))
                .unwrap_or(file);

            let mut sorted_snippet_ids: Vec<_> = snippet_ids.iter().cloned().collect();

            sorted_snippet_ids.sort();

            for snippet_id in sorted_snippet_ids {
                println!(" - {}, snippet id: {}", path_view.display(), snippet_id);
            }
        }
    }
}
