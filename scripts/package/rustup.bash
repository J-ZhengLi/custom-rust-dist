#!/usr/bin/bash
# only the rustup-init binary is needed
echo "packaging for $1 with version $2, to dir: $3"

_output="$OUTPUT_DIR/rustup/archive/$2/$TARGET"
mkdir -p $_output

_bin="$1-init"
[[ "$TARGET" == *"windows"* ]] && _bin="$1-init.exe"

cp ./target/$TARGET/release/$_bin $_output

cd $_output
sha256sum $_bin > "$_bin.sha256"
