# Removing a Prefix from Snippets

The `strip_prefix` attribute allows you to remove a specified string from the beginning of each line in the snippet. 
This is useful when you need to format the injected snippets by stripping out comment prefixes or other unwanted characters.

## Example
Let's demonstrate how to use the `strip_prefix` attribute to remove the prefix from each line of the `example-strip-prefix` snippet.

### Original Snippet in Code:
```
<!-- <snip id="example-strip-prefix"> -->
//! Snipdoc
//!
//! ## Installation
//! 
//! ```sh
//! cargo install
//! ```
<!-- </snip> -->
```
### Stripping Prefix from Snippet:
To remove the prefix from each line (e.g., //!), use the `strip_prefix` attribute like this:

<!-- <snip id="example-strip-prefix" inject_from="code" strip_prefix="//!"> -->
 Snipdoc

 ## Installation
 
 ```sh
 cargo install
 ```
<!-- </snip> -->