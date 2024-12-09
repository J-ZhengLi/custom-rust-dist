#!/usr/bin/env bash

set -e

# 安装 Rust
sh ci/scripts/install-rust.sh

# 安装 nodejs 18.x
sh ci/scripts/install-nodejs.sh

# 安装 pnpm
npm set strict-ssl false && npm install -g pnpm

# 安装 Tauri CLI
cargo install tauri-cli@1.6.4

echo "Execiting cargo dev vendor"
cargo dev vendor

echo "Execiting cargo dev dist"
cargo dev dist