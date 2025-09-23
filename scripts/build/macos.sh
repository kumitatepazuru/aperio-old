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

# src-tauriにbinariesディレクトリを作成
mkdir -p src-tauri/binaries

# GStreamerの必要なバイナリをコピー
cp -R "$BIN"/* src-tauri/binaries/

# pythonの共有ライブラリのパスを取得
PYTHON_LIB=$(python3 -c "import sysconfig; print(sysconfig.get_config_var('LDLIBRARY'))")
PYTHON_PATH=$(python3 -c "import sysconfig; print(sysconfig.get_config_var('LIBDIR'))")
if [ -z "$PYTHON_LIB" ]
then
  echo "Failed to get Python shared library name"
  exit 1
fi
if [ -z "$PYTHON_PATH" ]
then
  echo "Failed to get Python library path"
  exit 1
fi
if [ ! -f "$PYTHON_PATH/$PYTHON_LIB" ]
then
  echo "Python shared library not found: $PYTHON_PATH/$PYTHON_LIB"
  exit 1
fi
# ファイルがリンクの可能性があるので実体になるまでたどる
while [ -L "$PYTHON_PATH/$PYTHON_LIB" ]; do
  LINK_TARGET=$(readlink "$PYTHON_PATH/$PYTHON_LIB") # 例：libpython3.13.dylib.1.0 -> libpython3.13.dylib.1.0.1
  if [[ "$LINK_TARGET" = /* ]]; then # 絶対パスかどうか
    PYTHON_LIB="$LINK_TARGET"
  else
    PYTHON_LIB="$(dirname "$PYTHON_PATH/$PYTHON_LIB")/$LINK_TARGET"
  fi
done
echo "Python shared library: $PYTHON_PATH/$PYTHON_LIB"

# src-tauri/binariesにコピー
cp "$PYTHON_PATH/$PYTHON_LIB" src-tauri/binaries/
