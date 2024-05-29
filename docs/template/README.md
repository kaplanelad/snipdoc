# Template

The `template` attribute allows you to wrap your snippet with a given template. This is useful when you have a snippet that you want to format in a specific way, such as wrapping it with a YAML tag format.

## Example
Let's demonstrate how to use the template attribute to wrap a snippet in a YAML format.


### Original Snippet in Yaml file:
```
<!-- <snip id="example-template"> -->
snippets:
  test:
    content: test
    path: main.rs

<!-- </snip> -->
```
### Applying a Template:
To wrap the snippet with a YAML code block, use the `template` attribute like this:
 
<!-- <snip id="example-template" inject_from="code" template="```yaml\n{snippet}\n```"> -->
```yaml
snippets:
  test:
    content: test
    path: main.rs

```
<!-- </snip> -->