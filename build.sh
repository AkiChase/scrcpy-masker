SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if [[ "$(uname)" == "Darwin" ]]; then
    echo "Building for MacOS arm64"
    PREFIX="ffmpeg-macos"
    OS="macos-arm64"
elif [[ "$(uname)" == "Linux" ]]; then
    echo "Building for Linux x64"
    PREFIX="ffmpeg-linux"
    OS="linux-x64"
else
    echo "Unhandled system: $(uname). Exiting."
    exit 1
fi

export PKG_CONFIG_PATH="$SCRIPT_DIR/ffmpeg-7.1.2/$PREFIX/lib/pkgconfig"
export FFMPEG_DIR="$SCRIPT_DIR/ffmpeg-7.1.2/$PREFIX"
export DYLD_LIBRARY_PATH="$SCRIPT_DIR/assets/lib/$OS:$DYLD_LIBRARY_PATH"

if [[ "$1" == "run" ]]; then
    cargo run
elif [[ "$1" == "release" ]]; then
    cargo build --release
else
    echo "Usage: $0 {run|release}"
    exit 1
fi