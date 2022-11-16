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

curdir="$(pwd)"
export CACHE_DIR="$curdir/cache"
export OUTPUT_DIR="$curdir/output"
export PATCHES_DIR="$curdir/patches"
export SCRIPTS_DIR="$curdir/scripts"
# ensure these directories exist
mkdir -p $CACHE_DIR
mkdir -p $OUTPUT_DIR

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
    MANIFEST_PATH="$CACHE_DIR/channel-rust-$TOOLCHAIN.toml"
    if [[ "$1" == "force" || ! -f $MANIFEST_PATH ]]; then
        wget -q -O $MANIFEST_PATH $_manifest_url
    fi
}

clone_and_build() {
    # build rustup first
    _cmd="git clone $RUSTUP_GIT --depth 1 $CACHE_DIR/rustup"
    eval "$_cmd"
    # fetch version
    get_rustup_version_from_source "$CACHE_DIR/rustup"
    [[ -z $RUSTUP_VERSION || "$RUSTUP_VERSION" == "" ]] && err "unable to get rustup version"
    # apply patch then build with specified script
    pack_for_tool "rustup" $RUSTUP_VERSION "rustup"

    for tool in "${EXTRA_TOOLS_ARGS[@]}"; do
        IFS=';'
        read -ra tool_info <<< "$tool"
        tool_name=${tool_info[0]}
        tool_ver=${tool_info[1]}
        tool_git=${tool_info[2]}
        _dir_name=${tool_info[3]:=$tool_name}

        # clone source into specific directory under cache
        _cmd="git clone $tool_git --depth 1 $CACHE_DIR/$_dir_name"
        eval "$_cmd"

        tools_cloned+=("$tool_name;$tool_ver")

        # apply patch if has
        pack_for_tool $tool_name $tool_ver $_dir_name
        
    done
}

# Apply patch, build, and finally generagte a package tarball.
# 
# args:
#   - name: actual name of the tool
#   - version: version of the tool
#   - dirname: git repo name
pack_for_tool() {
    _dir_name=${3:-$1}
    cd "$CACHE_DIR/$_dir_name"
    [[ ! "$?" == "0" ]] && err "no source code found for '$1'"

    # patch
    if [ -d "$PATCHES_DIR/$1" ]; then
        for pf in "$(ls $PATCHES_DIR/$1)"; do
            patch -p0 -N < $PATCHES_DIR/$1/$pf
        done
    fi

    # build
    if [ -f "$SCRIPTS_DIR/build/$1.bash" ]; then
        bash $SCRIPTS_DIR/build/$1.bash
    else
        cd $curdir
        err "no build script for $1 found, exiting..."
    fi

    # pack as tarball
    if [ -f "$SCRIPTS_DIR/package/$1.bash" ]; then
        _pkg_dir="$CACHE_DIR/$1-$TARGET"
        mkdir -p $_pkg_dir
        bash $SCRIPTS_DIR/package/$1.bash $1 $2 $_pkg_dir
    fi
    cd $curdir
}

get_rust_target() {
    _host=$(rustc -vV | grep 'host')
    [[ ! "$?" == "0" || -z $_host ]] && err "unable to get rust target, please check rustc installation"
    IFS=' '
    _outputs=($_host)
    export TARGET=${_outputs[1]}
}

# args:
#   source_code_dir - Path to rustup source code
get_rustup_version_from_source() {
    [[ ! -f "$1/Cargo.toml" ]] && err "directory '$1' does not contains Cargo.toml"

    RUSTUP_VERSION=$(grep -m 1 'version' $1/Cargo.toml | grep -o '".*"' | sed 's/"//g')
}

test() {
    get_rust_target
    ensure_manifest
    clone_and_build
}

test