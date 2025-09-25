use anyhow::{anyhow, Context, Ok, Result};
use std::fs;

use tauri::{AppHandle, Manager};
use tauri_plugin_shell::ShellExt;

use crate::app_config::read_config;

pub fn get_base_args(appdata_dir: &str) -> Vec<&str> {
    vec!["--directory", appdata_dir, "--no-cache"]
}

fn file_extension(s: &str) -> String {
    // windowsならexeをいれて返却
    if cfg!(target_os = "windows") {
        return format!("{}.exe", s);
    }
    s.to_string()
}

async fn run_uv(app: &AppHandle, args: Vec<&str>) -> Result<String> {
    let cmd = app.shell().sidecar(file_extension("uv"))?.args(args);

    let output = cmd.output().await?;
    if !output.status.success() {
        return Err(anyhow!(
            "Command failed with status: {}",
            String::from_utf8(output.stderr.clone())?
        ));
    }

    Ok(String::from_utf8(output.stdout.clone())?)
}

pub fn check_python_installed(app: &AppHandle) -> bool {
    // appdataのdir pathを取得
    let appdata_dir = app.path().app_data_dir().unwrap();
    // python/bin/python(.exe)のpathを取得
    let python_path = appdata_dir
        .join("python")
        .join("bin")
        .join(file_extension("python"));
    println!("Checking for Python at path: {:?}", python_path);

    // pythonが存在するか確認
    if python_path.exists() {
        println!("Python is already installed at {:?}", python_path);
        return true;
    }
    false
}

pub async fn install_packages(app: &AppHandle, packages: Vec<&str>) -> Result<()> {
    // appdataのdir pathを取得
    let appdata_path = app.path().app_data_dir()?;
    let appdata_dir = appdata_path.to_str().unwrap();

    let python_dir = appdata_path.join("python");
    let mut args = vec!["add"];
    args.extend(packages);
    args.extend([
        "--no-python-downloads",
        "--python",
        python_dir.to_str().unwrap(),
    ]);
    args.extend(get_base_args(appdata_dir));

    run_uv(app, args).await?;
    Ok(())
}

pub async fn install_python(app: &AppHandle) -> Result<()> {
    // appdataのdir pathを取得
    let appdata_path = app.path().app_data_dir()?;
    let appdata_dir = appdata_path.to_str().unwrap();
    let config = read_config(app).context("No config found, cannot install Python")?;

    // uv initをする
    let python_version = format!(
        ">={},<{}",
        config.python.version, config.python.next_version
    );
    let mut args = vec![
        "init",
        "--python",
        python_version.as_str(),
        "--bare",
        "--author-from",
        "none",
        "--name",
        "aperio-env",
    ];
    args.extend(get_base_args(appdata_dir));
    run_uv(app, args).await?;

    // uv python installコマンドを実行してpythonをインストール
    args = vec![
        "python",
        "install",
        "--no-bin",
        "--install-dir",
        appdata_dir,
        "--project",
        appdata_dir,
        &config.python.version,
    ];
    args.extend(get_base_args(appdata_dir));
    run_uv(app, args).await?;

    // 何故かゴミができるのであれば削除
    fs::remove_file(appdata_path.join(".gitignore")).ok();
    fs::remove_file(appdata_path.join(".lock")).ok();
    fs::remove_dir(appdata_path.join(".temp")).ok();

    // cpythonから始まるディレクトリができるので、pythonにリネーム
    let cpython_dir = fs::read_dir(&appdata_path)?
        .filter_map(|entry| entry.ok())
        .find(|entry| {
            let binding = entry.file_name();
            let file_name = binding.to_string_lossy();
            file_name.starts_with("cpython")
        })
        .map(|entry| entry.path())
        .context("No cpython directory found in app data dir")?;

    fs::rename(cpython_dir, appdata_path.join("python")).ok();

    // resources/wheelディレクトリの中からopencv-python-headlessのwhlファイルを探す
    let wheel_dir = app.path().resource_dir()?.join("wheels");
    let wheel_path = std::fs::read_dir(wheel_dir)?
        .filter_map(|entry| entry.ok())
        .find(|entry| {
            let binding = entry.file_name();
            let file_name = binding.to_string_lossy();
            file_name.starts_with("opencv_python_headless") && file_name.ends_with(".whl")
        })
        .map(|entry| entry.path())
        .context("No opencv-python-headless wheel file found in resources/wheel")?;

    // uv addコマンドを実行してopencv-python-headlessをインストール
    match install_packages(app, vec![wheel_path.to_str().unwrap()]).await {
        std::result::Result::Ok(_) => {
            println!("Successfully installed Python and required packages")
        }
        Err(_) => return Err(anyhow!("Failed to install required packages")),
    }

    Ok(())
}

pub async fn sync_packages(app: &AppHandle) -> Result<String> {
    let appdata_path = app.path().app_data_dir()?;
    let appdata_dir = appdata_path.to_str().unwrap();

    let mut args = vec!["sync"];
    args.extend(get_base_args(appdata_dir));

    Ok(run_uv(app, args).await?)
}
