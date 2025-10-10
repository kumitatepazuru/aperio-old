$url = "https://gstreamer.freedesktop.org/data/pkg/windows/1.26.6/msvc/gstreamer-1.0-devel-msvc-x86_64-1.26.6.msi"
$runtimeUrl = "https://gstreamer.freedesktop.org/data/pkg/windows/1.26.6/msvc/gstreamer-1.0-msvc-x86_64-1.26.6.msi"

$out = "$env:RUNNER_TEMP\gstreamer.msi"
curl.exe -L "$url" -o "$out"
Write-Host "Saved to $out"

$msi = "$env:RUNNER_TEMP\gstreamer.msi"
Start-Process msiexec.exe -ArgumentList "/i `"$msi`" /qn /norestart ADDLOCAL=ALL INSTALLDIR=C:\gstreamer\" -Wait -NoNewWindow
$gstRoot = "C:\gstreamer\1.0\msvc_x86_64"
$gstBin  = Join-Path $gstRoot "bin"
$gstLib  = Join-Path $gstRoot "lib"
if (!(Test-Path $gstBin)) { throw "Not found: $gstBin" }

# runtimeもインストール
$out = "$env:RUNNER_TEMP\gstreamer-runtime.msi"
curl.exe -L "$runtimeUrl" -o "$out"
Write-Host "Saved to $out"

$msi = "$env:RUNNER_TEMP\gstreamer-runtime.msi"
Start-Process msiexec.exe -ArgumentList "/i `"$msi`" /qn /norestart ADDLOCAL=ALL INSTALLDIR=C:\gstreamer-runtime\" -Wait -NoNewWindow
$gstRuntimeBin = "C:\gstreamer-runtime\1.0\msvc_x86_64\bin"

tree /f $gstRuntimeBin

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
uv sync --locked --all-extras --dev
uv run scripts/update-uv.py

# src-tauriにbinariesディレクトリを作成(なければ)
New-Item -ItemType Directory -Path src-tauri\binaries -Force

# GStreamerの必要なバイナリをコピー
Copy-Item -Path "$gstRuntimeBin\*.dll" -Destination "src-tauri\binaries\" -Force -Verbose

# pythonの共有ライブラリのパスを取得
$pythonPath = (python -c "import sys; print(sys.exec_prefix)").Trim()
# バージョンを取得
$pythonVersion = (python -c "import sys; print(f'{sys.version_info.major}.{sys.version_info.minor}')" ).Trim()
# DLLのパスを作成(バージョンのドットを削除)
$pythonDllPath = Join-Path $pythonPath "python$($pythonVersion -replace '\.','').dll"
if (!(Test-Path $pythonDllPath)) { throw "Not found: $pythonDllPath" }
# DLLをコピー
Copy-Item -Path $pythonDllPath -Destination "src-tauri\binaries\" -Force -Verbose

# opencvをビルド
# "\"がCMakeの引数でエスケープされてしまうので、"/"に置換しておく
$gstLib  = $gstLib -replace '\\','/'
$gstRoot = $gstRoot -replace '\\','/'

# https://medium.com/@kenancan.dev/building-opencv-gstreamer-on-windows-a-8-hour-battle-bdb3211aa834
# https://qiita.com/TakahiroOta/items/a34b3d1db6475ddc31d7
$env:CMAKE_ARGS = "-DGSTREAMER_app_LIBRARY=${gstLib}/gstapp-1.0.lib `
  -DGSTREAMER_audio_LIBRARY=${gstLib}/gstaudio-1.0.lib `
  -DGSTREAMER_base_LIBRARY=${gstLib}/gstbase-1.0.lib `
  -DGSTREAMER_glib_INCLUDE_DIR=${gstRoot}/include/glib-2.0 `
  -DGSTREAMER_glib_LIBRARY=${gstLib}/glib-2.0.lib `
  -DGSTREAMER_glibconfig_INCLUDE_DIR=${gstLib}/glib-2.0/include `
  -DGSTREAMER_gobject_LIBRARY=${gstLib}/gobject-2.0.lib `
  -DGSTREAMER_gst_INCLUDE_DIR=${gstRoot}/include/gstreamer-1.0 `
  -DGSTREAMER_gstreamer_LIBRARY=${gstLib}/gstreamer-1.0.lib `
  -DGSTREAMER_pbutils_LIBRARY=${gstLib}/gstpbutils-1.0.lib `
  -DGSTREAMER_riff_LIBRARY=${gstLib}/gstriff-1.0.lib `
  -DGSTREAMER_video_LIBRARY=${gstLib}/gstvideo-1.0.lib"


bash scripts/build/opencv/opencv.sh

# github actions用: PKG_CONFIG系はGIHUB_ENVに保存して他のステップで使えるようにする
Add-Content -Path $env:GITHUB_ENV -Value "PKG_CONFIG_PATH=$env:PKG_CONFIG_PATH"
Add-Content -Path $env:GITHUB_ENV -Value "PATH=$env:PATH"