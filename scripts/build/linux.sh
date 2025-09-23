sudo apt-get update
sudo apt-get install -y libwebkit2gtk-4.1-dev \
  libappindicator3-dev \
  librsvg2-dev \
  patchelf \
  libgstreamer1.0-dev \
  libgstreamer-plugins-base1.0-dev \
  gstreamer1.0-plugins-base \
  gstreamer1.0-plugins-good \
  gstreamer1.0-plugins-bad \
  gstreamer1.0-plugins-ugly \
  --no-install-recommends --no-install-suggests

which pkg-config || true
pkg-config --modversion gstreamer-1.0 || { echo "pkg-config can't find gstreamer-1.0"; exit 1; }

bun install --frozen-lockfile

# src-tauriにbinariesディレクトリを作成
mkdir -p src-tauri/binaries

# pythonの共有ライブラリのパスを取得
PYTHON_PATH=$(python3 -c "import sysconfig; print(sysconfig.get_config_var('LIBDIR'))")
PYTHON_LIB=$(python3 -c "import sys; print(f'libpython{sys.version_info.major}.{sys.version_info.minor}.so')")

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
while [ -L "$PYTHON_LIB" ]; do
  LINK_TARGET=$(readlink "$PYTHON_LIB") # 例：libpython3.13.so.1.0 -> libpython3.13.so.1.0.1
  if [[ "$LINK_TARGET" = /* ]]; then # 絶対パスかどうか
    PYTHON_LIB="$LINK_TARGET"
  else
    PYTHON_LIB="$(dirname "$PYTHON_LIB")/$LINK_TARGET"
  fi
done
echo "Python shared library: $PYTHON_LIB"

# src-tauri/binariesにコピー
cp --verbose "$PYTHON_LIB" src-tauri/binaries/
