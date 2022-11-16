#!/usr/bin/bash
echo "packaging for $1 with version $2, to dir: $3"

tool_root="$3"
tool_dir="$tool_root/$1"
mkdir -p $tool_dir

_cli_bin="$1-cli"
_web_bin="$1-web"
[[ "$TARGET" == *"windows"* ]] && _cli_bin="$1-cli.exe" && _web_bin="$1-web.exe"

mkdir -p $tool_dir/bin
cp ./target/release/$_cli_bin $tool_dir/bin
cp ./target/release/$_web_bin $tool_dir/bin

# Copy license & document info
cp -r ./*.md $tool_root
_doc_dir="$tool_dir/share/doc/$1"
mkdir -p $_doc_dir
cp -r ./*.md $_doc_dir

# write manifest.in
echo "file:bin/$_cli_bin" > $tool_dir/manifest.in
echo "file:bin/$_web_bin" >> $tool_dir/manifest.in
echo "dir:share/doc/$1" >> $tool_dir/manifest.in

# write components
echo "$1" > $tool_root/components
# write version
echo "$2" > $tool_root/version
# write rust-installer-version
echo "3" > $tool_root/rust-installer-version

cd $CACHE_DIR
tool_root_full="$(basename $tool_root)"
tar -cJf $tool_root_full.tar.xz $tool_root_full/
sha256sum -t $tool_root_full.tar.xz > $tool_root_full.tar.xz.sha256
mv $tool_root.tar.* $OUTPUT_DIR/
