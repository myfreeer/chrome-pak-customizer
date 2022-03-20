
# PowerShell.exe -ExecutionPolicy Bypass -File ci-download.ps1

$download_base="https://github.com/myfreeer/chrome-pak-customizer/releases/download/1.0/"

$test_files=@(
"chrome_100_percent_99.0.4844.51-1.pak"
"chrome_200_percent_99.0.4844.51-1.pak"
"en-US_99.0.4844.51-1.pak"
"resources_99.0.4844.51-1.pak"
"test_v4_474896_1.pak"
"test_v4_474896_2.pak"
"test_v4_474896_r.pak"
"test_v5_550886_1.pak"
"test_v5_550886_2.pak"
"test_v5_550886_r.pak"
"zh-CN_99.0.4844.51-1.pak"
)

New-Item -Path "test_dir" -ItemType Directory -Force

foreach ($item in $test_files)
{
    if (-not (Test-Path ("test_dir\" + $item)))
    {
        (New-Object Net.WebClient).DownloadFile($download_base + $item, "test_dir\" + $item)
    }
}
