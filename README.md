# multilint

A tool to run multiple linters.

## Usage

```
$ cargo install multilint
$ multilint --help
multilint 0.1.2
A driver of multiple linters

USAGE:
    multilint [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --config <config>      Config file [default: multilint.toml]
    -p, --printer <printer>    Message format [default: text]  [possible values: Null, Text]
    -C <work-dir>              Run linters at the directory [default: .]
```

## Configuration format

`multilint.toml` should exist in the working directory.

### Example

```
[global]
excludes = ["third_party/**"]

[linter.shell]
command = "shellcheck"
options = ["--external-sources", "--source-path=SCRIPTDIR"]
includes = ["*.sh"]
excludes = ["*.zsh"]

[linter.clippy]
command = "cargo"
options = ["clippy"]
work_dir = "subdir"
```

## Related projects

- [treefmt](https://github.com/numtide/treefmt): multilint is inspired by treefmt
