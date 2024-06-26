Expected to inject `readme` from the code and strip the prefix `//!`
<!-- <snip id="readme" inject_from="code" strip_prefix="//!"> -->
<!-- </snip> -->


Expected to inject `title` from the code and strip the prefix `//!`
<!-- <snip id="title" inject_from="code" strip_prefix="//!"> --><!-- </snip> -->

Expected to inject `rust-print` from the code 
```rust
<!-- <snip id="rust-print" inject_from="code"> -->

<!-- </snip> -->
```

# Usage commands
Expected to inject `create-db` from the code 
<!-- <snip id="create-db" inject_from="code"> -->
  
<!-- </snip> -->

Expected to skip injection as the content of the snippet is the same
<!-- <snip id="inject-snippets" inject_from="code"> -->
  ```sh
    snipdoc run
  ```
<!-- </snip> -->

Expected to ignore this section since the `not-found-snippet-to-inject` snippet id does not exist
<!-- <snip id="not-found-snippet-to-inject" inject_from="code"> -->
<!-- </snip> -->


Expected to inject only `inject-from-yaml` snippet id from the YAML snippets configuration
<!-- <snip id="inject-from-yaml" inject_from="yaml"> -->
<!-- </snip> -->

Expected to skip injections since `inject_from` is configured to inject from the code and not from the YAML
<!-- <snip id="inject-from-yaml" inject_from="code"> -->
<!-- </snip> -->


Expected to inject only `inject-from-yaml` snippet id from the YAML. the `inject_from` value is `any` which means if snippet id found in one of the data source it injected
<!-- <snip id="inject-from-yaml" inject_from="any"> -->
<!-- </snip> -->

Expected to skip inject, see in `snipdoc-config.yml` file that the path of this snippet is excluded from the run
<!-- <snip id="should-ignore" inject_from="code"> -->

<!-- </snip> -->


Expected to inject the content of the snippet with the given template
<!-- <snip id="config-template" inject_from="code" template="```yaml \n {snippet} \n ```"> -->

<!-- </snip> -->

Expected to inject the content of the snippet with the pre-defined template
<!-- <snip id="config-template" inject_from="code" template="yaml"> -->

<!-- </snip> -->


<!-- <snip id="rust-print" inject_from="code" template="wrap_impl"> -->
```rust

```
<!-- </snip> -->

Expected to inject the content of the snippet with the given template with a different comment tag
# <snip id="config-template" inject_from="code" template="```yaml \n {snippet} \n ```"> 

# </snip> 
