#!/usr/bin/env bash

rm -rf test_out
mkdir -p test_out

is_success=1

for file in ./test_dir/*.pak
do
  file_path="test_dir/${file}"
  out_dir="test_out/${file}_out"
  out_index="${out_dir}/pak_index.ini"
  out_file="test_out/${file}_repack.pak"
  cargo run --release u "$file_path" "$out_dir"
  cargo run --release p "$out_index" "$out_file"
  file_hash=$(sha256sum "$file_path" | cut -f1 -d ' ')
  out_hash=$(sha256sum "$out_file" | cut -f1 -d ' ')
  if ! [ $file_hash = $out_hash ]; then
      echo "file ${file} fail"
      is_success=0
  fi
done

if [ $is_success = 0 ]; then
    exit 1
fi
