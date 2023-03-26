# lf_lint

Linter that checks that all files in PATH ends with a newline.
Optionally adds a newline where missing.

```
$ lf_lint --help
Checks that all text files in <PATH> ends with a newline

Usage: lf_lint [OPTIONS] --path <PATH>

Options:
  -p, --path <PATH>  Path to operate on
  -f, --fix          Add new line at EOF if missing
  -h, --help         Print help
  -V, --version      Print version
```

This tool is available as a [Crate](https://crates.io/crates/lf_lint) and as a [Docker image](https://hub.docker.com/r/andni233/lf_lint).
