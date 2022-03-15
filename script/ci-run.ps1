New-Item -Path "test_out" -ItemType Directory -Force

$is_success=1

foreach ($file in get-ChildItem test_dir\*) {
    $file_path = "test_dir\" + $file.name
    $out_dir = "test_out\" + $file.name + "_out"
    $out_index = $out_dir + "\pak_index.ini"
    $out_file = "test_out\" + $file.name + "_repack.pak"
    cargo run --release u $file_path $out_dir
    cargo run --release p $out_index $out_file
    if ((Get-FileHash $file_path).Hash -ne (Get-FileHash $out_file).Hash) {
        echo ("file " + $file.name + " fail")
        $is_success=0
    }
}

if ($is_success -eq 0) {
    exit 1
}
