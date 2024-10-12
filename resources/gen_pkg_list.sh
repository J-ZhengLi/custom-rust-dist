set -x

RUST_VERSION='1.81.0'
DIST_DATE='2024-09-05'
SERVER='http://static.rust-lang.org'

RUSTUP_VERSION='1.27.1'

manifest='channel-rust-stable.toml'
targets=(
    x86_64-pc-windows-gnu
    x86_64-pc-windows-msvc
    x86_64-unknown-linux-gnu
)
targeted_components=(
    cargo-$RUST_VERSION
    clippy-$RUST_VERSION
    llvm-tools-$RUST_VERSION
    rustc-$RUST_VERSION
    rustc-dev-$RUST_VERSION
    rust-docs-$RUST_VERSION
    rustfmt-$RUST_VERSION
    rust-std-$RUST_VERSION
)
other_components=(
    rust-mingw-$RUST_VERSION-x86_64-pc-windows-gnu
    rust-src-$RUST_VERSION
)
other_tools=(
    x86_64-pc-windows-gnu/x86_64-14.2.0-release-posix-seh-ucrt-rt_v12-rev0.7z@https://github.com/niXman/mingw-builds-binaries/releases/download/14.2.0-rt_v12-rev0/x86_64-14.2.0-release-posix-seh-ucrt-rt_v12-rev0.7z
)

write_dist() {
    pl=packages.txt
    echo "dist" > $pl # truncate
    echo "dist/$DIST_DATE" >> $pl
    for cp in ${targeted_components[@]}; do
        for target in ${targets[@]}; do
            pkg=$cp-$target.tar.xz
            sha=$pkg.sha256
            echo "dist/$DIST_DATE/$pkg@$SERVER/dist/$pkg" >> $pl
            echo "dist/$DIST_DATE/$sha@$SERVER/dist/$sha" >> $pl
        done
    done
    for cp in ${other_components[@]}; do
        echo "dist/$DIST_DATE/$cp.tar.xz@$SERVER/dist/$cp.tar.xz" >> $pl
        echo "dist/$DIST_DATE/$cp.tar.xz.sha256@$SERVER/dist/$cp.tar.xz.sha256" >> $pl
    done
    echo "dist/$manifest@$SERVER/dist/$manifest" >> $pl
    echo "dist/$manifest.sha256@$SERVER/dist/$manifest.sha256" >> $pl
}

write_rustup() {
    for target in ${targets[@]}; do
        echo "$target" >> $pl

        path=$SERVER/rustup/archive/$RUSTUP_VERSION/$target
        [[ $target == *windows* ]] && rustup=rustup-init.exe || rustup=rustup-init
        echo "$target/$rustup@$path/$rustup" >> $pl
    done
}

write_tools() {
    for tool in ${other_tools[@]}; do
        echo $tool >> $pl
    done
}

write_dist
write_rustup
write_tools
