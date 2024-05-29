# Running Snipdoc

To inject snippets from your code or YAML file, follow these steps:

## Identify the Snippet:

### Directly from the code
Identify the piece of content in your code that you want to grab and wrap the snippet with the following tag:
```txt
// <snip id="SNIPPET_ID_FROM_CODE">
pub fn print() {
println!("Snipdoc");
}
// </snip>
```

You can hide the snippet from presentation using various comment styles:
- In Markdown files (like readme.md), use HTML comments: <!-- CONTENT HERE -->
- In Rust file docs, use: `//! CONTENT HERE`
- In Rust function docs, use: `/// CONTENT HERE`
- Use appropriate comment tags for other file formats.

### Directly from yaml
Create a snipdoc.yml file with the following structure:
```yaml
snippets:
  SNIPPET_ID_FROM_YAML:
    content: |
      cargo install snipdoc
    path: ./snipdoc.yml

```

## Copying the Snippet:

To copy the snippet, use `inject_from="code"` or  `inject_from="code"` (depend of the data source). This will copy the snippet with the same ID and paste it into the destination.

### From Code:
```
<!-- <snip id="SNIPPET_ID_FROM_CODE" inject_from="code"> -->
pub fn print() {
println!("Snipdoc");
}
<!-- </snip> -->
```

### From Yaml:
```
<!-- <snip id="SNIPPET_ID_FROM_YAML" inject_from="yaml"> -->
cargo install snipdoc
<!-- </snip> -->
```

## Inject
After construction the snippet rust the following command:
```sh
snipdoc run
```
