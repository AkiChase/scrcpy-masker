Write-Host "Building for Windows x64"

$PREFIX = "ffmpeg-windows"
$DYLIB = "windows-x64"

$SCRIPT_DIR = Get-Location
$env:PKG_CONFIG_PATH = "$SCRIPT_DIR\ffmpeg-7.1.2\$PREFIX\lib\pkgconfig"
$env:FFMPEG_DIR = "$SCRIPT_DIR\ffmpeg-7.1.2\$PREFIX"
$env:PATH = "$SCRIPT_DIR\assets\lib\$DYLIB;$env:PATH"

if ($args[0] -eq "run") {
    cargo run
} elseif ($args[0] -eq "release") {
    cargo build --release
} else {
    Write-Host "Usage: .\script.ps1 {run|release}"
    exit 1
}
