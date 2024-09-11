# RIM-Dev

Helper commands for development

## How to use

### Debug with manager mode (GUI)

```bash
cargo dev run-manager
```

### Debug with manager mode (CLI)

```bash
cargo dev run-manager --cli
```

check for more manager-mode help

```bash
cargo dev run-manager --cli --help
```

### Generate release binaries

```bash
cargo dev dist
```

### Set name of the vendor

this will affect the binary name, package identifier, default install dir, and every output containing the `vendor` key in [translation file](../locales/en.json) etc.

```bash
cargo dev set-vendor <NAME>
```

i.e. If the vendor name was set to `my-rust`, the final binaries will be `my-rust-installer` and `my-rust-installer-cli`.

### Other

for more functionalities, check `--help`

```bash
cargo dev --help
```
