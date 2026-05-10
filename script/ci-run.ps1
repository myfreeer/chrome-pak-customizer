New-Item -Path "test_out" -ItemType Directory -Force

$is_success=1
$bin_path=".\target\release\chrome-pak-customizer.exe"

cargo build --release

function Check-RoundTrip($file_path, $out_dir, $out_file, $unpack_arg, $pack_arg) {
    $out_index = $out_dir + "\pak_index.ini"
    & $bin_path $unpack_arg $file_path $out_dir
    & $bin_path $pack_arg $out_index $out_file
    if ((Get-FileHash $file_path).Hash -ne (Get-FileHash $out_file).Hash) {
        echo ("file " + $file_path + " fail")
        $script:is_success=0
    } else {
        echo ("file " + $file_path + " passed")
    }
}

foreach ($file in get-ChildItem test_dir\*) {
    $file_path = "test_dir\" + $file.name
    $out_dir = "test_out\" + $file.name + "_out"
    $out_file = "test_out\" + $file.name + "_repack.pak"
    Check-RoundTrip $file_path $out_dir $out_file "-u" "-p"
    if ($file.name -eq "msedge_100_percent.pak") {
        Check-RoundTrip `
            $file_path `
            ("test_out\" + $file.name + "_edge_out") `
            ("test_out\" + $file.name + "_edge_repack.pak") `
            "-ue" `
            "-pe"
        Check-RoundTrip `
            $file_path `
            ("test_out\" + $file.name + "_mmap_out") `
            ("test_out\" + $file.name + "_mmap_repack.pak") `
            "-um" `
            "-p"
    }
}

if ($is_success -eq 0) {
    exit 1
}
