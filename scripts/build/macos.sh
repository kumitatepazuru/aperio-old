# runtimeとdevelの両方が必要
R_URL="https://gstreamer.freedesktop.org/data/pkg/osx/1.26.6/gstreamer-1.0-1.26.6-universal.pkg"       # runtime
D_URL="https://gstreamer.freedesktop.org/data/pkg/osx/1.26.6/gstreamer-1.0-devel-1.26.6-universal.pkg" # devel
curl -L "$R_URL" -o "$RUNNER_TEMP/gst-runtime.pkg"
curl -L "$D_URL" -o "$RUNNER_TEMP/gst-devel.pkg"

sudo installer -pkg "$RUNNER_TEMP/gst-runtime.pkg" -target /
sudo installer -pkg "$RUNNER_TEMP/gst-devel.pkg"   -target /

BIN="/Library/Frameworks/GStreamer.framework/Versions/1.0/bin"
[ -d "$BIN" ] || { echo "Not found: $BIN"; exit 1; }

# pkg-configにパスを通す
# PATH 先頭に GStreamer bin を置く（: 区切り）
echo "PATH=$BIN${PATH:+:$PATH}" >> "$GITHUB_ENV"

# pkg-config のパス（lib/share 両方）
echo "PKG_CONFIG_PATH=/Library/Frameworks/GStreamer.framework/Versions/1.0/lib/pkgconfig:/Library/Frameworks/GStreamer.framework/Versions/1.0/share/pkgconfig${PKG_CONFIG_PATH:+:$PKG_CONFIG_PATH}" >> "$GITHUB_ENV"

# Intel mac: sysroot が未設定だと失敗することがあるので明示
echo "PKG_CONFIG_SYSROOT_DIR=/" >> "$GITHUB_ENV"

# verify
which pkg-config || true
pkg-config --modversion gstreamer-1.0 || (echo "pkg-config can't find gstreamer-1.0" && exit 1)

bun install --frozen-lockfile
bun run tauri build # 一回ビルドをしてなんのlibraryが必要かを確認する