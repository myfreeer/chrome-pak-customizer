#!/usr/bin/env bash

rm -rf test_out
mkdir -p test_out

set -e
is_success=1
bin_path="target/release/chrome-pak-customizer"

cargo build --release

check_round_trip() {
  local file_path="$1"
  local out_dir="$2"
  local out_file="$3"
  local unpack_arg="$4"
  local pack_arg="$5"
  local out_index="${out_dir}/pak_index.ini"
  "$bin_path" "$unpack_arg" "$file_path" "$out_dir"
  "$bin_path" "$pack_arg" "$out_index" "$out_file"
  file_hash=$(sha256sum "$file_path" | cut -f1 -d ' ')
  out_hash=$(sha256sum "$out_file" | cut -f1 -d ' ')
  if ! [ "$file_hash" = "$out_hash" ]; then
    echo "file ${file_path} fail"
    is_success=0
  else
    echo "file ${file_path} passed"
  fi
}

for file in ./test_dir/*.pak
do
  file_name=$(basename "$file")
  file_path="test_dir/${file_name}"
  out_dir="test_out/${file_name}_out"
  out_file="test_out/${file_name}_repack.pak"
  check_round_trip "$file_path" "$out_dir" "$out_file" -u -p
  if [ "$file_name" = "msedge_100_percent.pak" ]; then
    check_round_trip \
      "$file_path" \
      "test_out/${file_name}_edge_out" \
      "test_out/${file_name}_edge_repack.pak" \
      -ue \
      -pe
    check_round_trip \
      "$file_path" \
      "test_out/${file_name}_mmap_out" \
      "test_out/${file_name}_mmap_repack.pak" \
      -um \
      -p
  fi
done

if [ $is_success = 0 ]; then
    exit 1
fi
