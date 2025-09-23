$url = "https://gstreamer.freedesktop.org/data/pkg/windows/1.26.6/msvc/gstreamer-1.0-devel-msvc-x86_64-1.26.6.msi"
$out = "$env:RUNNER_TEMP\gstreamer.msi"
curl.exe -L "$url" -o "$out"
Write-Host "Saved to $out"

$msi = "$env:RUNNER_TEMP\gstreamer.msi"
Start-Process msiexec.exe -ArgumentList "/i `"$msi`" /qn /norestart ADDLOCAL=ALL INSTALLDIR=C:\gstreamer\" -Wait -NoNewWindow
$gstRoot = "C:\gstreamer\1.0\msvc_x86_64"
$gstBin  = Join-Path $gstRoot "bin"
if (!(Test-Path $gstBin)) { throw "Not found: $gstBin" }

tree /f $gstRoot

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

# src-tauriにbinariesディレクトリを作成(なければ)
New-Item -ItemType Directory -Path src-tauri\binaries -Force

# GStreamerの必要なバイナリをコピー
Copy-Item -Path "$gstRoot\lib\*.dll" -Destination "src-tauri\binaries\" -Force -Verbose

# pythonの共有ライブラリのパスを取得
$pythonPath = (python -c "import sys; print(sys.exec_prefix)").Trim()
# バージョンを取得
$pythonVersion = (python -c "import sys; print(f'{sys.version_info.major}.{sys.version_info.minor}')" ).Trim()
# DLLのパスを作成(バージョンのドットを削除)
$pythonDllPath = Join-Path $pythonPath "python$($pythonVersion -replace '\.','').dll"
if (!(Test-Path $pythonDllPath)) { throw "Not found: $pythonDllPath" }
# DLLをコピー
Copy-Item -Path $pythonDllPath -Destination "src-tauri\binaries\" -Force -Verbose