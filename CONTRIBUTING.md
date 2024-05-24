# Contributing code to Snipdoc

Snipdoc is open source and we love to receive contributions from our community — you! There are many ways to contribute, from improving the documentation, submitting bug reports and feature requests or writing code.

## How to contribute

The preferred and easiest way to contribute changes to the project is to fork it on GitHub, and then create a pull request to ask us to pull your changes into our repo. We use GitHub's pull request workflow to review the contribution, and either ask you to make any refinements needed or merge it and make them ourselves.

Your PR must also:

 - be based on the `main` branch
 - adhere to the [code style](#code-style)
 - pass the [test suites](#tests)
 - check [documentation](#documentation)
 - add new [patterns](./docs/add-new-patterns.md)

 Your Issue need to contains:
 - Version information
 - Steps to reproduce
 - Snippet example


## Tests

In `Snipdoc` we have few test suite flows that need to pass before merging to master.
- [unitest](#unitest)
- [clippy](#clippy)
- [rustfmt](#rustfmt)

### unitest
To capture the snapshots test we using [trycmd](https://crates.io/crates/trycmd) crate. if you PR contains parser changes, make sure you add/update the snippet and update the snapshots
```bash
TRYCMD=overwrite cargo test
```

### clippy
```bash
cargo clippy --all-features -- -D warnings -W clippy::pedantic -W clippy::nursery -W rust-2018-idioms
```

### rustfmt
```bash
cargo fmt --all -- --check
```

## Code style

We use the standard Rust code style, and enforce it with `rustfmt`/`cargo fmt`.
A few code style options are set in the [`.rustfmt.toml`](./.rustfmt.toml) file, and some of them are not stable yet and require a nightly version of rustfmt.


## documentation

Generate and open [Snipdoc](https://github.com/kaplanelad/snipdoc) to make sure that your documentation current

```bash
cargo doc --open
```