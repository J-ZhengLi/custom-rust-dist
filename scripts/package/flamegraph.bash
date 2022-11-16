#!/usr/bin/bash
echo "packaging for $1 with version $2, to dir: $3"

tool_root="$3"
tool_dir="$tool_root/$1"
mkdir -p $tool_dir

_bin="$1"
_bin_cargo="cargo-$1"
[[ "$TARGET" == *"windows"* ]] && _bin="$1.exe" _bin_cargo="cargo-$1.exe"

mkdir -p $tool_dir/bin
cp ./target/release/$_bin $tool_dir/bin
cp ./target/release/$_bin_cargo $tool_dir/bin

# Copy license & document info
cp -r ./*.md $tool_root
cp -r ./LICENSE* $tool_root
_doc_dir="$tool_dir/share/doc/$1"
mkdir -p $_doc_dir
cp -r ./*.md $_doc_dir
cp -r ./LICENSE* $_doc_dir

# write manifest.in
echo "file:bin/$_bin" > $tool_dir/manifest.in
echo "file:bin/$_bin_cargo" >> $tool_dir/manifest.in
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
