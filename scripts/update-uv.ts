import jszip from "jszip";
import * as tar from "tar";
import fs from "fs";
import zlib from "zlib";
import path from "path";
import { Readable } from "stream";
import { ReadableStream } from "stream/web";
import { pipeline } from "stream/promises";

const UV_LINK = "https://github.com/astral-sh/uv/releases/latest/download";
const FILES = [
  "uv-x86_64-unknown-linux-gnu.tar.gz",
  "uv-aarch64-unknown-linux-gnu.tar.gz",
  "uv-x86_64-apple-darwin.tar.gz",
  "uv-x86_64-pc-windows-msvc.zip",
  "uv-aarch64-pc-windows-msvc.zip",
];

// ファイルをダウンロードしつつ解凍(UV_LINK/FILE)
// リダイレクトは勝手に追跡してもらう
async function downloadAndDecompress(
  url: string,
  outputPath: string,
  decompressedDir: string
) {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`Failed to download file: ${response.statusText}`);
  }
  if (!response.body) {
    throw new Error("Response body is null");
  }

  const ext = url.includes("windows") ? ".exe" : "";

  // zipとtar.gzで場合分け
  if (url.endsWith(".zip")) {
    const buffer = await response.arrayBuffer();
    const zip = await jszip.loadAsync(buffer);
    // uv(.exe)だけ抽出して保存
    const fileName = Object.keys(zip.files).find(
      (name) => name.endsWith("uv") || name.endsWith("uv.exe")
    );
    if (!fileName) {
      throw new Error("uv file not found in the zip archive");
    }
    const fileData = await zip.files[fileName].async("nodebuffer");
    fs.writeFileSync(decompressedDir + `/uv${ext}`, fileData);
  } else if (url.endsWith(".tar.gz")) {
    const responseStream = response.body;
    if (!responseStream) {
      throw new Error("Response body is null");
    }
    const gunzip = zlib.createGunzip();
    await pipeline(
      Readable.fromWeb(responseStream as unknown as ReadableStream<any>),
      gunzip,
      tar.extract({
        cwd: decompressedDir, // 出力先ディレクトリ
        filter: (path) => {
          // uv(.exe)だけ抽出
          return path.endsWith("uv") || path.endsWith("uv.exe");
        },
        strip: 1, // 最初のディレクトリを無視
      })
    ).catch((err) => {
      console.error("Pipeline failed:", err);
      throw err;
    });
  }

  // ファイルのos名のとおりにリネーム
  const urlName = path.basename(url).split(".")[0].slice(3); // 拡張子なしのファイル名(uv-を除く)
  const finalOutputPath = `${outputPath}/uv-${urlName}${ext}`;
  fs.renameSync(decompressedDir + `/uv${ext}`, finalOutputPath);
  console.log(`Downloaded and decompressed to ${finalOutputPath}`);
}

async function main() {
  const outputPath = `src-tauri/binaries/uv`;
  fs.mkdirSync(outputPath, { recursive: true });

  const decompressedDir = outputPath + "/tmp";
  if (!fs.existsSync(decompressedDir)) {
    fs.mkdirSync(decompressedDir, { recursive: true });
  }

  for (const file of FILES) {
    const fileUrl = `${UV_LINK}/${file}`;
    await downloadAndDecompress(fileUrl, outputPath, decompressedDir);
  }

  // 一時ディレクトリを削除
  fs.rmdirSync(decompressedDir, { recursive: true });
  console.log("All files downloaded and decompressed successfully.");
}

main().catch((error) => {
  console.error("Error:", error);
  process.exit(1);
});
