#!/bin/bash

mkdir ./cargo_tauri

if [ "$(uname -s)" = "Linux" ]; then
    if [ "$(uname -m)" = "x86_64" ]; then
        curl -OL https://github.com/LuuuXXX/tauri-mirror/releases/download/tauri-cli-v1.6.3/cargo-tauri-x86_64-unknown-linux-gnu.tgz
        tar -xf cargo-tauri-x86_64-unknown-linux-gnu.tgz -C ./cargo_tauri
    else
        curl -OL https://github.com/LuuuXXX/tauri-mirror/releases/download/tauri-cli-v1.6.3/cargo-tauri-aarch64-unknown-linux-gnu.tgz
        tar -xf cargo-tauri-aarch64-unknown-linux-gnu.tgz -C ./cargo_tauri
    fi
    mv ./cargo_tauri/cargo-tauri ~/.cargo/bin
else
    curl -OL https://github.com/LuuuXXX/tauri-mirror/releases/download/tauri-cli-v1.6.3/cargo-tauri-x86_64-pc-windows-msvc.zip
    unzip cargo-tauri-x86_64-pc-windows-msvc.zip -d ./cargo_tauri
    mv ./cargo_tauri/cargo-tauri.exe ~/.cargo/bin
fi

rm -r ./cargo_tauri