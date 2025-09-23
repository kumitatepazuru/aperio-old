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
