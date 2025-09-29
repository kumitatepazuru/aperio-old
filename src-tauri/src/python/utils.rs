use anyhow::{anyhow, Context, Ok, Result};
use pyo3::prelude::PyAnyMethods;
use pyo3::Python;
use std::{env, fs};
use tauri::{AppHandle, Manager};

use crate::dir_util::get_data_dir;
use crate::dir_util::get_local_data_dir;
use tauri_plugin_shell::ShellExt;
use toml_edit::DocumentMut;

pub struct PythonStatus {
    pub installed: bool,
    pub version: Option<String>,
}

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

pub fn add_python_path_env(app: &AppHandle) -> Result<()> {
    // PYTHONPATHとPYTHONHOMEの設定
    let local_data_dir = get_local_data_dir(&app)?; // 環境ファイルがある
    let resource_dir = app.path().resource_dir()?; // 本体がある
    let appdata_dir = get_data_dir(&app)?; // ユーザーデータがある
    let python_path = local_data_dir.join("python"); // pythonがある

    env::set_var(
        "PYTHONPATH",
        env::join_paths([&python_path, &resource_dir, &appdata_dir])?,
    );
    env::set_var("PYTHONHOME", &python_path);
    println!("Set PYTHONPATH and PYTHONHOME to {:?}", &python_path);

    // Python::initialize();
    // println!("Python interpreter initialized.");

    Ok(())
}
pub async fn check_python_installed(app: &AppHandle) -> Result<PythonStatus> {
    // appdataのdir pathを取得
    let appdata_dir = get_local_data_dir(app)?;
    // python/bin/python(.exe)のpathを取得
    let python_path = appdata_dir
        .join("python")
        .join("bin")
        .join(file_extension("python"));
    println!("Checking for Python at path: {:?}", python_path);

    // pythonが存在するか確認
    if !python_path.exists() {
        println!("Python executable not found at {:?}", python_path);
        return Ok(PythonStatus {
            installed: false,
            version: None,
        });
    }

    // pythonのversionを取得
    // libpythonとvenv(uvが作った環境)のバージョンが全て合わなければCライブラリ系が読み込めないっぽい?
    // https://github.com/axnsan12/drf-yasg/issues/362#issuecomment-515542184
    let python_version = Python::attach(|py| -> Result<String> {
        let sys = py.import("sys")?;
        let version = sys.getattr("version_info")?;
        let major: i32 = version.get_item(0)?.extract()?;
        let minor: i32 = version.get_item(1)?.extract()?;
        let micro: i32 = version.get_item(2)?.extract()?;
        println!("Embed Python version: {}", sys.getattr("version")?);

        Ok(format!("{}.{}.{}", major, minor, micro))
    })?;

    // インストールされているpythonのversionを取得
    let installed_python_version = String::from_utf8(
        app.shell()
            .command(&python_path)
            .args([
                "-c",
                "import sys; v=sys.version_info; print(f'{v[0]}.{v[1]}.{v[2]}')",
            ])
            .output()
            .await?
            .stdout,
    )?;
    let installed_python_version = installed_python_version.trim().to_string(); // 改行を削除

    if installed_python_version != python_version {
        println!(
            "Python version mismatch: expected(embed libpython) {}, found(installed python) {}. Try reinstalling.",
            python_version, installed_python_version
        );
        // ディレクトリ(.venv含む)を削除
        fs::remove_dir_all(appdata_dir.join("python")).ok();
        fs::remove_dir_all(appdata_dir.join(".venv")).ok();
        return Ok(PythonStatus {
            installed: false,
            version: Some(python_version),
        });
    }

    Ok(PythonStatus {
        installed: true,
        version: Some(python_version),
    })
}

pub async fn install_packages(app: &AppHandle, packages: Vec<&str>) -> Result<()> {
    // appdataのdir pathを取得
    let appdata_path = get_local_data_dir(app)?;
    let appdata_dir = appdata_path
        .to_str()
        .context("Failed to convert appdata path to str")?;

    let python_dir = appdata_path.join("python");
    let mut args = vec!["add"];
    args.extend(packages);
    args.extend([
        "--no-python-downloads",
        "--python",
        python_dir
            .to_str()
            .context("Failed to convert python path to str")?,
    ]);
    args.extend(get_base_args(appdata_dir));

    run_uv(app, args).await?;
    Ok(())
}

pub async fn install_python(app: &AppHandle, python_version: &str) -> Result<()> {
    // appdataのdir pathを取得
    let appdata_path = get_local_data_dir(app)?;
    let appdata_dir = appdata_path
        .to_str()
        .context("Failed to convert appdata path to str")?;

    // uv initをする
    // pyproject.tomlがあれば、手動で変更する
    let python_version_str = format!("~={}", python_version);
    let appdata_toml = appdata_path.join("pyproject.toml");
    if appdata_toml.exists() {
        let pj_data = fs::read_to_string(&appdata_toml)?;
        let mut pj_data = pj_data.parse::<DocumentMut>()?;
        pj_data["project"]["requires-python"] = toml_edit::value(&python_version_str);
        fs::write(&appdata_toml, pj_data.to_string())?;
        println!(
            "Updated pyproject.toml with requires-python = {}",
            &python_version_str
        );
    } else {
        let mut args = vec![
            "init",
            "--python",
            python_version_str.as_str(),
            "--bare",
            "--author-from",
            "none",
            "--name",
            "aperio-env",
        ];
        args.extend(get_base_args(appdata_dir));
        run_uv(app, args).await?;
    }

    // uv python installコマンドを実行してpythonをインストール
    let mut args = vec![
        "python",
        "install",
        "--no-bin",
        "--install-dir",
        appdata_dir,
        "--project",
        appdata_dir,
        python_version,
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
    let wheel_path = fs::read_dir(wheel_dir)?
        .filter_map(|entry| entry.ok())
        .find(|entry| {
            let binding = entry.file_name();
            let file_name = binding.to_string_lossy();
            file_name.starts_with("opencv_python_headless") && file_name.ends_with(".whl")
        })
        .map(|entry| entry.path())
        .context("No opencv-python-headless wheel file found in resources/wheel")?;

    // uv addコマンドを実行してopencv-python-headlessをインストール
    install_packages(
        app,
        vec![wheel_path
            .to_str()
            .context("could not convert wheel path to str")?],
    )
    .await?;
    println!("Successfully installed Python and required packages");

    Ok(())
}

pub async fn sync_packages(app: &AppHandle) -> Result<String> {
    let appdata_dir = get_local_data_dir(app)?;
    let appdata_dir = appdata_dir
        .to_str()
        .context("Failed to convert appdata path to str")?;

    let mut args = vec!["sync"];
    args.extend(get_base_args(appdata_dir));

    Ok(run_uv(app, args).await?)
}
