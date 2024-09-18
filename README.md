<div align="center">
  <img src="./media/logo.png"/>
  <h1>Snipdoc</h1>
  <h3>Managing documentation that includes code snippets</h3>

  [![crate](https://img.shields.io/crates/v/snipdoc.svg)](https://crates.io/crates/snipdoc)
  [![docs](https://docs.rs/snipdoc/badge.svg)](https://docs.rs/snipdoc)
</div>

**Snipdoc** is a straightforward tool for managing documentation that includes code snippets.
It collects snippets from your code or YAML files and injects them into various parts of your documentation, making the documentation process more efficient.

## Installation
To install Snipdoc, you can use Cargo:
```sh
cargo install snipdoc
```

Or download it from the [GitHub Repository.](https://github.com/kaplanelad/snipdoc/releases/latest)

## Getting Started
To collect and replace all snippets from multiple data sources, use:
```sh
snipdoc run
```
For a detailed guide on how it works, follow this [guide](./docs/inject/)
 
### Inject Options
Snipdoc provides several attributes to customize snippet injection:

#### Adding a Prefix to Snippets
The `add_prefix` attribute allows you to prepend a specified string to each line of the snippet. 
This is useful when you need to format the injected snippets with a specific prefix, such as for comments in code blocks.

[Check out this example](./docs/add_prefix)

#### Removing a Prefix from Snippets
The `strip_prefix` attribute allows you to remove a specified string from the beginning of each line in the snippet. 
This is useful when you need to format the injected snippets by stripping out comment prefixes or other unwanted characters.

[Check out this example](./docs/strip_prefix/)

#### Using Templates
The `template` attribute allows you to wrap your snippet with a given template. This is useful when you have a snippet that you want to format in a specific way, such as wrapping it with a YAML tag format.

[Check out this example](./docs/template/)

#### Executing Snippet Content
**Note:** For security reasons, this feature is turned off by default. To enable it, compile Snipdoc with the `exec` feature: `cargo install snipdoc --features exec`.

The `execute` action option allows you to run a snippet as a shell command and collect the output into a snippet. This is useful for adding the result of a `--help` command to your documentation, ensuring it stays up-to-date with your CLI tool's output, even if it changes.

Before executing the snippet command, you will be prompted for approval. This allows you to review and approve the command before execution.

To skip the approval prompt, set the environment variable `SNIPDOC_SKIP_EXEC_COMMANDS=true`.


[Check out this example](./docs/execute_snippet_content/)


### Managing Snippets

To manage all snippets effectively, run:
```sh
snipdoc show
```

### Creating a YAML File

You can mix snippets from your code with a YAML file configuration. Create an empty snipdoc.yml file by running:

Create an `snipdoc.yml` file by running the command:
   ```sh
   snipdoc create-db --empty
   ```


### Checking Snippets

Validate that all snippets are valid and match the current injected versions. This is useful for CI workflows to ensure documentation accuracy and consistency.

```sh
snipdoc check
```

#### Github Action
To integrate Snipdoc with GitHub Actions, use the following workflow configuration:


```yaml
name: docs

jobs:
  check:
    name: Check
    runs-on: ubuntu-latest
    permissions:
      contents: read
    steps:
      - name: Checkout the code
        uses: actions/checkout@v4
      - uses: actions-rs/toolchain@v1
        with:
          profile: minimal
          toolchain: stable
          override: true
      - run: cargo install snipdoc        
      - run: snipdoc check        
```
