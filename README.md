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
    -c, --config <config>      Config file [default: multilint.toml]
    -p, --printer <printer>    Message format [default: text]  [possible values: Null, Text,
                               JSONL]
    -C <work-dir>              Run linters at the directory [default: .]
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
work_dir = "subdir"
```

## Error message parser

You can define `formats` at config and use JSONL printer.

```
$ cat multilint.toml
[linter.shellcheck]
command = "shellcheck"
options = ["--format=gcc"]
includes = ["*.sh"]
formats = ["^%f:%l:%c: %m$"]
$ echo 'a=`pwd`' >test.sh
$ multilint
Running shellcheck ... failed
./test.sh:1:1: error: Tips depend on target shell and yours is unknown. Add a shebang or a 'shell' directive. [SC2148]
./test.sh:1:1: warning: a appears unused. Verify use (or export if used externally). [SC2034]
./test.sh:1:3: note: Use $(...) notation instead of legacy backticks `...`. [SC2006]
$ multilint -p jsonl
{"program":"shellcheck","file":"./test.sh","line":1,"column":1,"message":"error: Tips depend on target shell and yours is unknown. Add a shebang or a 'shell' directive. [SC2148]"}
{"program":"shellcheck","file":"./test.sh","line":1,"column":1,"message":"warning: a appears unused. Verify use (or export if used externally). [SC2034]"}
{"program":"shellcheck","file":"./test.sh","line":1,"column":3,"message":"note: Use $(...) notation instead of legacy backticks `...`. [SC2006]"}
```

## Related projects

- [treefmt](https://github.com/numtide/treefmt): multilint is inspired by treefmt
