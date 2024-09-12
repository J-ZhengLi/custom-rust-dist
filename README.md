# R.I.M (Rust Installation Manager)

Manage Rust toolchain and a set of extra tools with ease.

## How does it work?

This simple tool takes advantage of [`rustup`](https://github.com/rust-lang/rustup), while offering some configurations allowing users to manage not only the rust toolchain, but also extra tools that may or may not built with Rust.

## Usage

This program has seperated main functions, one is `installer` and one is `manager`.
The `manager` binary is basically a renamed version of `installer` which will be written locally to the user machine after successully running `installer`.

> this is a similar procedure done in rustup with its `rustup-init` and `rustup`.

### Install

Run the executable as `./installer [OPTIONS]`

```console
Options:
  -v, --verbose        Enable verbose output
  -q, --quiet          Suppress non-critical messages
  -y, --yes            Disable interaction and answer 'yes' to all prompts
      --prefix <PATH>  Set another path to install Rust
  -h, --help           Print help
  -V, --version        Print version
```

### Manage your installation

Run the executable as `manager [OPTIONS] [COMMAND]`

```console
Commands:
  uninstall  Uninstall individual components or everything
  try-it     A subcommand to create a new Rust project template and let you start coding with it
  help       Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose  Enable verbose output
  -q, --quiet    Suppress non-critical messages
  -y, --yes      Disable interaction and answer 'yes' to all prompts
  -h, --help     Print help
  -V, --version  Print version
```

1. uninstall selected tools:

```bash
./manager uninstall tool [TOOLS]
```

2. uninstall everything:

```bash
./manager uninstall all
```

3. Export a pre-configured example project for you to try Rust:

```bash
./manager try-it -p /path/to/create/project
```
