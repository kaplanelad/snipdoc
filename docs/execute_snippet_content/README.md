# Execute snippet content

The `execute` action option allows you to execute a snippet as a shell command and collect the output of the command into a snippet. 
This is useful when you want to add the result of a `--help` command to your documentation, ensuring that your documentation stays up-to-date with your CLI tool's output, even if it changes.

## Example
Let's demonstrate how to use the action="exec" attribute.

### Original Snippet in Code:
<!-- <snip id="help-command"> -->
snipdoc --help
<!-- </snip> -->

### Executing the Command:
To execute the command and capture its output, add `action="exec"` to your injection snippet:

<!-- <snip id="help-command" inject_from="code" action="exec"> -->
Code Documentation Made Simple

Usage: snipdoc [OPTIONS] [PATH] <COMMAND>

Commands:
  create-db  Create a local DB file
  check      Validate if snippets are equal, errors or missing configuration
  run        Inject snippet into placeholders
  show       Show snippets
  help       Print this message or the help of the given subcommand(s)

Arguments:
  [PATH]  Source code directory for collecting documentation [default: .]

Options:
  -l, --log-level <LOG_LEVEL>  Log level [default: INFO]
  -h, --help                   Print help
  -V, --version                Print version

<!-- </snip> -->