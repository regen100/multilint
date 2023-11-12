# multilint

[![Status](https://img.shields.io/github/actions/workflow/status/regen100/multilint/rust.yml)](https://github.com/regen100/multilint/actions/workflows/rust.yml)
[![Crates.io](https://img.shields.io/crates/v/multilint)](https://crates.io/crates/multilint)
[![License](https://img.shields.io/github/license/regen100/multilint)](https://github.com/regen100/multilint/blob/main/LICENSE)

A tool to run multiple linters.

## Usage

    $ cargo install multilint
    $ multilint --help
    multilint 0.1.3
    A driver of multiple linters

    USAGE:
        multilint [OPTIONS]

    FLAGS:
        -h, --help       Prints help information
        -V, --version    Prints version information

    OPTIONS:
        -f, --format <format>    Message format [default: text]  [possible values: Null, Raw,
                                 Text]
        -C <work-dir>            Changes the working directory before running

## Configuration format

All `multilint.toml` in directories from the root to the current directory are merged and parsed.

### Example

```toml
[global]
excludes = ["third_party/**"]

[linter.shellcheck]
command = "shellcheck"
includes = ["*.sh"]
excludes = ["*.zsh"]

[linter.clippy]
command = "cargo"
options = ["clippy"]
work_dir = "subdir"  # you can change directory

[linter.rustfmt]
command = "cargo"
options = ["fmt", "--"]  # formatters can be used as linters (mtime of the files are checked if `check_hash` is false)
includes = ["*.rs"]
```

## Related projects

*   [treefmt](https://github.com/numtide/treefmt): multilint is inspired by treefmt
