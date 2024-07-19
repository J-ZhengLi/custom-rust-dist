# Custom Rust Distro

Manage Rust toolchain and a set of extra tools with ease.

## How does it work?

This simple tool takes advantage of [`rustup`](https://github.com/rust-lang/rustup), while offering some configurations allowing users to manage not only the rust toolchain, but also extra tools that may or may not built with Rust.

## Usage

Run the executable as `./installer [OPTIONS] [COMMAND]`

```console
Commands:
  install    Install rustup, rust toolchain, and various tools
  uninstall  Uninstall individual components or everything
  help       Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose  Enable verbose output
  -q, --quiet    Suppress non-critical messages
  -y, --yes      Disable interaction and answer 'yes' to all prompts
  -h, --help     Print help
  -V, --version  Print version
```

### Install a set of tools

TBD

### Uninstallation

1. uninstall selected tools:

```bash
./installer uninstall tool [TOOLS]
```

2. uninstall everything:

```bash
./installer uninstall all
```
