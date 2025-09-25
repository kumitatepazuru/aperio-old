use anyhow::Result;
use pyo3::{types::PyAnyMethods, Python};
use tauri::{App, Manager};

use crate::{app_config, python};

pub fn startup(app: &App) -> Result<()> {
    tauri::async_runtime::block_on(async move {
        // configの初期化
        app_config::init_config(app);

        // pythonがインストールされているか確認
        let app_handle = app.handle().clone();
        if !python::utils::check_python_installed(&app_handle) {
            println!("Python is not installed. Installing...");
            let python_installed = python::utils::install_python(&app_handle).await;
            println!("Python installed: {:?}", python_installed);
        } else {
            println!("syncing packages...");
            let sync_result = python::utils::sync_packages(&app_handle).await;
            println!("Package sync result: {:?}", sync_result);
        }

        // PYTHONPATHとPYTHONHOMEの設定
        let appdata_dir = app.path().app_data_dir()?;
        let python_path = appdata_dir.join("python");

        std::env::set_var("PYTHONPATH", &python_path);
        std::env::set_var("PYTHONHOME", &python_path);
        println!("Set PYTHONPATH and PYTHONHOME to {:?}", &python_path);

        Python::initialize();
        println!("Python interpreter initialized.");

        // printテスト
        Python::attach(|py| {
            let sys = py.import("sys").expect("Failed to import sys module");
            let version: String = sys
                .getattr("version")
                .expect("Failed to get version")
                .extract()
                .expect("Failed to extract version");
            println!("Python version: {}", version);
        });

        Ok(())
    })
}
