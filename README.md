# Snipdoc

**Snipdoc** is a straightforward tool for managing documentation that includes code snippets.
It collects snippets from your code or YAML files and injects them into various parts of your documentation, making the documentation process more efficient.

## Installation
#### Cargo install:
```sh
cargo install snipdoc
```
Or download from the [GitHub Repository.](https://github.com/kaplanelad/snipdoc/releases/latest)
## Getting started:

### Collecting Snippets
To collect snippets, follow these steps:
1. Identify the piece of content in your code that you want to grab.
2. Wrap the snippet with the following tag:
```text
// <snip id="SNIPPET_ID">
     CONTENT HERE
// </snip>
```
3. You can hide the snippet from presentation using various comment styles:
    - In Markdown files (like readme.md), use HTML comments: `<!-- CONTENT HERE -->`
    - In Rust file docs, use: `//! CONTENT HERE`
    - In Rust function docs, use: `/// CONTENT HERE`
    - Use appropriate comment tags for other file formats.

### Injecting Snippets
To inject snippets, create an empty placeholder with the snippet ID and add `inject_from="code"` attribute like this:

```text
<!-- <snip id="SNIPPET_ID" inject_from="code"> -->
     CONTENT HERE
<!-- </snip> -->
```

After adding the placeholder, run the following command:

```sh
snipdoc run
```

#### Inject attributes
Following attributes are available when injecting the snippets as a attribute.

##### add_prefix
For adding a prefix for each snippet line use the `add_prefix` attribute. 

##### strip_prefix
For removing a prefix for each snippet line use `strip_prefix` attribute

##### template
Wrap the snippet content with a custom template use `template` attribute.


##### Execute snippet 
In case you want to inject content of command you can use `action="exec"`. this injection will run the snippet shell command and inject the output
Snippet:
```yaml
snippets:
  EXEC:
    content: echo '1+1'= $((1+1))
    path: ./snipdoc.yml
```

result:
```
<!-- <snip id="EXEC" inject_from="yaml" action="exec"> -->
1+1= 2

<!-- </snip> -->
```


### Managing Snippets

To manage all snippets effectively, run the following command:

```sh
snipdoc show
```

For a live example, run the following command inside your project:
```sh
 snipdoc show ./snipdoc/examples/inject
```

### Managing Snippets in YAML

If you prefer managing snippets in a YAML file, follow these steps:

1. Run the following command to create an empty `snipdoc.yml` file:
   ```sh
   snipdoc create-db --empty
   ```
2. Add your snippets to `snipdoc.yml`, and use them by injecting from YAML (`inject_from="code"`). 
For a live example, run the following command inside your project:
   ```sh
   snipdoc run ./snipdoc/examples/inject
   ```

### Check

validate that all snippets are valid and match the current injected versions. 
It is useful for incorporating into CI workflows to ensure documentation accuracy and consistency.

```sh
snipdoc check
```

#### Github Action
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
