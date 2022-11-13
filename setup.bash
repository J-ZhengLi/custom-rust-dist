#!/usr/bin/bash
#
# This script performs the following steps to fetch/generate files for distribution.
# 1. Download the official toolchain manifest from rust-lang offical
# 2. Clone and build extra tools, then pack them into tarballs
# 3. Clone, patch, build rustup
# 4. Create a copy of the official toolchain manifest, modify it as follow:
#   a. add extra tools as targeted rust extension for each extra tools
#   b. add extra tools definition
#   c. add extra tools in profiles
# 5. Move modified toolchain manifest, and modified rustup-init binary into output

TOOLCHAIN="stable"
OFFICIAL_RUST_DIST_SERVER="https://static.rust-lang.org"
DEFAULT_TARGET="x86_64-pc-windows-msvc"

curdir="$(pwd)"
CACHE="$curdir/cache"
PATCHES="$curdir/patches"
SCRIPTS="$curdir/scripts"

IFS=';'
# Git urls with args for cloning the tools
EXTRA_TOOLS_ARGS=(
    # grcov
    'grcov;0.8.13;https://github.com/mozilla/grcov.git -b v0.8.13'
    # rust-code-analysis
    'rust-code-analysis;0.0.24;https://github.com/mozilla/rust-code-analysis.git -b v0.0.24'
    # rustsec (cargo-audit)
    'cargo-audit;0.17.4;https://github.com/rustsec/rustsec.git -b cargo-audit/v0.17.4;rustsec'
    # rust-fuzz (cargo-fuzz)
    'cargo-fuzz;0.11.1;https://github.com/rust-fuzz/cargo-fuzz.git -b 0.11.1'
    # flamegraph-rs
    'flamegraph;0.6.2;https://github.com/flamegraph-rs/flamegraph.git'
    # crate type extensions are not applicable here
    # criterion.rs
    # mockall
    # libfuzzer-sys
)
# Rustup will be automatically patched and build during the proccess
RUSTUP_GIT='https://github.com/rust-lang/rustup.git -b stable'

tools_cloned=()

err() {
    _code=${2:-"1"}
    echo "ERROR: $1"
    exit $_code
}

ensure_manifest() {
    _manifest_url="$OFFICIAL_RUST_DIST_SERVER/dist/channel-rust-$TOOLCHAIN.toml"
    MANIFEST_PATH="$CACHE/channel-rust-$TOOLCHAIN.toml"
    if [[ "$1" == "force" || ! -f $MANIFEST_PATH ]]; then
        wget -q -O $MANIFEST_PATH $_manifest_url
    fi
}

clone_and_build() {
    # build rustup first
    echo "git clone $RUSTUP_GIT --depth 1 $CACHE/rustup"
    # apply patch then build with specified script
    patch_and_build "rustup"

    for tool in "${EXTRA_TOOLS_ARGS[@]}"; do
        read -ra tool_info <<< "$tool"
        tool_name=${tool_info[0]}
        tool_ver=${tool_info[1]}
        tool_git=${tool_info[2]}

        # clone source into specific directory under cache
        _dir_name=${tool_info[3]:=$tool_name}
        echo "git clone $tool_git --depth 1 $CACHE/$_dir_name"
        tools_cloned+=("$tool_name;$tool_ver")
        echo "${tools_cloned[@]}"

        # apply patch if has
        patch_and_build "$tool_name" $_dir_name
        
    done
}

# args:
#   - name: actual name of the tool
#   - dirname: git repo name
patch_and_build() {
    cd "$CACHE/$_dir_name"
    [[ ! "$?" == "0" ]] && err "no source code found for '$1'"

    # patch
    if [ -d "$PATCHES/$1" ]; then
        _dir_name=${2:-$1}
        for pf in "$(ls $PATCHES/$1)"; do
            echo "patch -p0 < $PATCHES/$1/$pf"
        done
    fi

    # build
    if [ -f "$SCRIPTS/build/$1.bash" ]; then
        echo "bash $SCRIPTS/build/$1.bash"
    else
        cd -
        err "ERROR: no build script for $1 found, exiting..."
    fi

    # pack as tarball

    cd -
}

test() {
    ensure_manifest
    echo $MANIFEST_PATH

    clone_and_build
}

test