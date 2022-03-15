#!/usr/bin/env bash
set -e
mkdir -p test_dir

test_files=(
 chrome_100_percent_99.0.4844.51-1.pak
 chrome_200_percent_99.0.4844.51-1.pak
 en-US_99.0.4844.51-1.pak
 resources_99.0.4844.51-1.pak
 test_v4_474896_1.pak
 test_v4_474896_2.pak
 test_v4_474896_r.pak
 test_v5_550886_1.pak
 test_v5_550886_2.pak
 test_v5_550886_r.pak
 zh-CN_99.0.4844.51-1.pak
)

download_base="https://github.com/myfreeer/chrome-pak-customizer/releases/download/1.0/"

for i in "${test_files[@]}"; do
  if [ ! -f "test_dir/${i}" ]; then
      curl --request GET -sL \
           --url "${download_base}${i}"\
           --output "test_dir/${i}"
  fi
done
