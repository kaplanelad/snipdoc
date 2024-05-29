# Adding a Prefix to Snippets

The `add_prefix` attribute allows you to prepend a specified string to each line of the snippet. 
This is useful when you need to format the injected snippets with a specific prefix, such as for comments in code blocks.

## Example
Let's say we have a snippet identified by `example-add-prefix` that we want to inject with a prefix for each line.

### Original Snippet in Code:
```
<!-- <snip id="example-add-prefix"> -->
# Snipdoc

## Installation

cargo install
<!-- </snip> -->
```

### Injecting Snippet with a Prefix:
To inject this snippet with a prefix (e.g., `// ` for comments), you use the `add_prefix` attribute like this:
 ```
<!-- <snip id="example-add-prefix" inject_from="code" add_prefix="// "> -->
// # Snipdoc
// 
// ## Installation
// 
// cargo install
<!-- </snip> -->
```