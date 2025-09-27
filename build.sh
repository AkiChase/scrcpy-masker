SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if [[ "$(uname)" == "Darwin" ]]; then
    echo "Building for MacOS"
    export PKG_CONFIG_PATH="$SCRIPT_DIR/ffmpeg-7.1.2/ffmpeg-macos/lib/pkgconfig"
    export FFMPEG_DIR="$SCRIPT_DIR/ffmpeg-7.1.2/ffmpeg-macos"
    export DYLD_LIBRARY_PATH="$SCRIPT_DIR/assets/lib/darwin-arm64:$DYLD_LIBRARY_PATH"
fi

if [[ "$1" == "run" ]]; then
    cargo run
elif [[ "$1" == "build" ]]; then
    cargo build
else
    echo "Usage: $0 {run|build}"
    exit 1
fi