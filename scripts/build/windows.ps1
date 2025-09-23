$url = "https://gstreamer.freedesktop.org/data/pkg/windows/1.26.6/msvc/gstreamer-1.0-devel-msvc-x86_64-1.26.6.msi"
$out = "$env:RUNNER_TEMP\gstreamer.msi"
curl.exe -L "$url" -o "$out"
Write-Host "Saved to $out"

$msi = "$env:RUNNER_TEMP\gstreamer.msi"
Start-Process msiexec.exe -ArgumentList "/i `"$msi`" /qn /norestart ADDLOCAL=ALL INSTALLDIR=C:\gstreamer\" -Wait -NoNewWindow
$gstRoot = "C:\gstreamer\1.0\msvc_x86_64"
$gstBin  = Join-Path $gstRoot "bin"
if (!(Test-Path $gstBin)) { throw "Not found: $gstBin" }

# PATH 先頭に GStreamer bin を置く（; 区切り）
$env:PATH = "$gstBin;$env:PATH"

# pkg-config の .pc 探索パスも設定
$pc1 = Join-Path $gstRoot "lib\pkgconfig"
$pc2 = Join-Path $gstRoot "share\pkgconfig"
$env:PKG_CONFIG_PATH = "$pc1;$pc2"

choco install -y pkgconfiglite

# verify
Get-Command pkg-config -ErrorAction SilentlyContinue
if ($LASTEXITCODE -ne 0) {
  Write-Host "pkg-config not found"
}

try {
  pkg-config --modversion gstreamer-1.0
  if ($LASTEXITCODE -ne 0) {
    throw "pkg-config can't find gstreamer-1.0"
  }
} catch {
  Write-Host "pkg-config can't find gstreamer-1.0"
  exit 1
}

bun install --frozen-lockfile
bun run tauri build # 一回ビルドをしてなんのlibraryが必要かを確認する
