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
export PATH=$BIN${PATH:+:$PATH}

# pkg-config のパス（lib/share 両方）
export PKG_CONFIG_PATH=/Library/Frameworks/GStreamer.framework/Versions/1.0/lib/pkgconfig:/Library/Frameworks/GStreamer.framework/Versions/1.0/share/pkgconfig${PKG_CONFIG_PATH:+:$PKG_CONFIG_PATH}

# Intel mac: sysroot が未設定だと失敗することがあるので明示
export PKG_CONFIG_SYSROOT_DIR=/

# verify
which pkg-config || true
pkg-config --modversion gstreamer-1.0 || { echo "pkg-config can't find gstreamer-1.0"; exit 1; }

bun install --frozen-lockfile
bun run uv:update

# src-tauriにbinariesディレクトリを作成
mkdir -p src-tauri/binaries

# GStreamerの必要なバイナリをコピー
cp -R -v "$BIN"/../lib/*.dylib src-tauri/binaries/

# pythonの共有ライブラリのパスを取得
PYTHON_PATH=$(python3 -c "import sysconfig; print(sysconfig.get_config_var('LIBDIR'))")
PYTHON_LIB=$(python3 -c "import sys; print(f'libpython{sys.version_info.major}.{sys.version_info.minor}.dylib')")

if [ -z "$PYTHON_PATH" ]
then
  echo "Failed to get Python library path"
  exit 1
fi
echo "Python library path: $PYTHON_PATH"
echo "ls $PYTHON_PATH:"
ls -alh "$PYTHON_PATH"

PYTHON_LIB="$PYTHON_PATH/$PYTHON_LIB"

# ファイルがリンクの可能性があるので実体になるまでたどる
echo "Python shared library: $PYTHON_LIB"

# src-tauri/binariesにコピー
cp -v "$PYTHON_LIB" src-tauri/binaries/

# opencvをビルド
bash scripts/build/opencv/opencv.sh