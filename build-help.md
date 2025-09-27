## FFMpeg

### Download
``` bash
# cd path/to/scrcpy-mask
wget https://ffmpeg.org/releases/ffmpeg-7.1.2.tar.bz2
tar -xjf ffmpeg-7.1.2.tar.bz2
rm ffmpeg-7.1.2.tar.bz2
cd ffmpeg-7.1.2
```

### Build

#### Windows

```bash
./configure --prefix=./ffmpeg-windows \
    --target-os=win64 --arch=x86_64 --toolchain=msvc \
    --enable-decoder=h264 --enable-decoder=hevc --enable-decoder=av1 \
    --enable-swscale --enable-filter=scale \
    --enable-avformat --enable-avcodec --enable-avutil --enable-swresample \
    --enable-gpl --disable-static --enable-shared
make -j$(nproc)
make install
```

#### MacOS

```bash
./configure --prefix=./ffmpeg-macos \
    --target-os=darwin --arch=arm64 \
    --enable-decoder=h264 --enable-decoder=hevc --enable-decoder=av1 \
    --enable-swscale --enable-filter=scale \
    --enable-avformat --enable-avcodec --enable-avutil --enable-swresample \
    --enable-videotoolbox \
    --enable-gpl --disable-static --enable-shared
make -j$(sysctl -n hw.ncpu)
make install
```

```bash
# TODO 将其他版本的写入build.sh 还有对应的 bat
export PKG_CONFIG_PATH=./ffmpeg-7.1.2/ffmpeg-macos/lib/pkgconfig
export FFMPEG_DIR=./ffmpeg-7.1.2/ffmpeg-macos
export DYLD_LIBRARY_PATH=./assets/lib/darwin-arm64:$DYLD_LIBRARY_PATH

cargo build --release
```