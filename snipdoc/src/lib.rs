//! <div align="center">
//!   <img src="https://github.com/kaplanelad/snipdoc/raw/main/media/logo.png>
//!   <h1>Snipdoc</h1>
//!   <h3>Managing documentation that includes code snippets</h3>
//!
//!   [![crate](https://img.shields.io/crates/v/snipdoc.svg)](https://crates.io/crates/snipdoc)
//!   [![docs](https://docs.rs/snipdoc/badge.svg)](https://docs.rs/snipdoc)
//! </div>
//!
//! **Snipdoc** is a straightforward tool for managing documentation that
//! includes code snippets. It collects snippets from your code or YAML files
//! and injects them into various parts of your documentation, making the
//! documentation process more efficient.
//!
//! ## Installation
//! To install Snipdoc, you can use Cargo:
//! ```sh
//! cargo install snipdoc
//! ```
//!
//! Or download it from the [GitHub Repository.](https://github.com/kaplanelad/snipdoc/releases/latest)
//!
//! ## Getting Started
//! To collect and replace all snippets from multiple data sources, use:
//! ```sh
//! snipdoc run
//! ```
//! For a detailed guide on how it works, follow this [guid](https://github.com/kaplanelad/snipdoc/tree/main/docs/inject)
//!  
//! ### Inject Options
//! Snipdoc provides several attributes to customize snippet injection:
//!
//! #### Adding a Prefix to Snippets
//! The `add_prefix` attribute allows you to prepend a specified string to each
//! line of the snippet. This is useful when you need to format the injected
//! snippets with a specific prefix, such as for comments in code blocks.
//!
//! [Check out this example](https://github.com/kaplanelad/snipdoc/tree/main/docs/add_prefix)
//!
//! #### Removing a Prefix from Snippets
//! The `strip_prefix` attribute allows you to remove a specified string from
//! the beginning of each line in the snippet. This is useful when you need to
//! format the injected snippets by stripping out comment prefixes or other
//! unwanted characters.
//!
//! [Check out this example](https://github.com/kaplanelad/snipdoc/tree/main/docs/strip_prefix/)
//!
//! #### Using Templates
//! The `template` attribute allows you to wrap your snippet with a given
//! template. This is useful when you have a snippet that you want to format in
//! a specific way, such as wrapping it with a YAML tag format.
//!
//! [Check out this example](https://github.com/kaplanelad/snipdoc/tree/main/docs/template/)
//!
//! #### Executing Snippet Content
//! The `execute` action option allows you to execute a snippet as a shell
//! command and collect the output of the command into a snippet. This is useful
//! when you want to add the result of a `--help` command to your documentation,
//! ensuring that your documentation stays up-to-date with your CLI tool's
//! output, even if it changes.
//!
//! [Check out this example](https://github.com/kaplanelad/snipdoc/tree/main/docs/execute_snippet_content/)
//!
//!
//! ### Managing Snippets
//!
//! To manage all snippets effectively, run:
//! ```sh
//! snipdoc show
//! ```
//!
//! ### Creating a YAML File
//!
//! You can mix snippets from your code with a YAML file configuration. Create
//! an empty snipdoc.yml file by running:
//!
//! Create an `snipdoc.yml` file by running the command:
//!    ```sh
//!    snipdoc create-db --empty
//!    ```
//!
//!
//! ### Checking Snippets
//!
//! Validate that all snippets are valid and match the current injected
//! versions. This is useful for CI workflows to ensure documentation accuracy
//! and consistency.
//!
//! ```sh
//! snipdoc check
//! ```
//!
//! #### Github Action
//! To integrate Snipdoc with GitHub Actions, use the following workflow
//! configuration:
//!
//!
//! ```yaml
//! name: docs
//!
//! jobs:
//!   check:
//!     name: Check
//!     runs-on: ubuntu-latest
//!     permissions:
//!       contents: read
//!     steps:
//!       - name: Checkout the code
//!         uses: actions/checkout@v4
//!       - uses: actions-rs/toolchain@v1
//!         with:
//!           profile: minimal
//!           toolchain: stable
//!           override: true
//!       - run: cargo install snipdoc        
//!       - run: snipdoc check        
//! ```

#[cfg(feature = "cli")]
pub mod cli;
pub mod config;
pub mod db;
pub mod errors;
pub mod parser;
mod read_file;
#[cfg(feature = "reporters")]
pub mod reporters;
#[cfg(test)]
pub mod tests_cfg;
pub mod walk;
