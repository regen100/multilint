# multilint

A tool to run multiple linters.

## Usage

```
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
    -c, --config <config>    Config file [default: multilint.toml]
    -f, --format <format>    Message format [default: text]  [possible values: Null, Text,
                             JSONL, GNU]
    -C <work-dir>            Changes the working directory before running
```

## Configuration format

`multilint.toml` should exist in the working directory.

### Example

```
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
options = ["fmt", "--"]  # formatters can be used as linters (mtime of the files are checked)
includes = ["*.rs"]
```

## Error message parser

You can define `formats` at config and use JSONL printer or GNU printer.

```
$ cat multilint.toml
[linter.shellcheck]
command = "shellcheck"
options = ["--format=gcc"]
includes = ["*.sh"]
formats = ["^%f:%l:%c: %m$"]

$ cat test.sh
#!/bin/sh
a=`pwd`

$ multilint
Running shellcheck ... failed
test.sh:2:1: warning: a appears unused. Verify use (or export if used externally). [SC2034]
test.sh:2:3: note: Use $(...) notation instead of legacy backticks `...`. [SC2006]

$ multilint -f jsonl
{"program":"shellcheck","file":"test.sh","line":2,"column":1,"message":"warning: a appears unused. Verify use (or export if used externally). [SC2034]"}
{"program":"shellcheck","file":"test.sh","line":2,"column":3,"message":"note: Use $(...) notation instead of legacy backticks `...`. [SC2006]"}

$ multilint -p gnu
shellcheck:test.sh:2:1: warning: a appears unused. Verify use (or export if used externally). [SC2034]
shellcheck:test.sh:2:3: note: Use $(...) notation instead of legacy backticks `...`. [SC2006]
```

## Related projects

- [treefmt](https://github.com/numtide/treefmt): multilint is inspired by treefmt
