# multilint

A [treefmt](https://github.com/numtide/treefmt)-inspired tool to run multiple linters.

## Configuration format
`multilint.toml` should exists in the working directory.

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
```
