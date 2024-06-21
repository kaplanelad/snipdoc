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
### Applying a pre defined Template:
To wrap the snippet with a YAML code block, use the `template` attribute like this and give `yaml` value for getting the following results:
 
<!-- <snip id="example-template" inject_from="code" template="yaml"> -->
```yaml
snippets:
  test:
    content: test
    path: main.rs

```

<!-- </snip> -->
### Applying a Custom Template:
Adding a custom template should contains any string in the template and for locate the injection add `{snippet}`.
 
<!-- <snip id="example-template" inject_from="code" template="```yaml\n{snippet}\n```"> -->
```yaml
snippets:
  test:
    content: test
    path: main.rs

```
<!-- </snip> -->


### Template from config
If you have a custom template that you want to reuse in multiple places, you can define it in a `snipdoc.yml` file located in the root folder. Alternatively, you can initialize an empty configuration by running the command `snipdoc create-db --empty`. The configuration file should follow this format:

```yaml
snippets:
  SNIPPET_ID:
    content: ''
    path: ./snipdoc.yml
templates:
  wrap_impl:
    content: |-
      ```rust
      impl test {
          {snippet}
      }
      ```
```
In this example:
- wrap_impl is the ID of your template.
- To use this template, specify its ID in the template attribute when injecting snippets, like this:


```
<!-- <snip id="example-template" inject_from="code" template="wrap_impl"> -->

<!-- </snip> -->
```

This setup allows you to easily reuse and maintain templates across your documentation. Adjust the `content` of the template as needed to suit your formatting requirements.






