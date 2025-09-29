use crate::dir_util::{get_data_dir, get_local_data_dir};
use anyhow::Result;
use pyo3::prelude::PyAnyMethods;
use pyo3::{PyResult, Python};
use tauri::AppHandle;

pub fn initialize_python(app: AppHandle) -> Result<()> {
    let appdata_dir = get_data_dir(&app)?;
    let local_data_dir = get_local_data_dir(&app)?;
    let venv_path = local_data_dir.join(".venv"); // venvがある

    // Pythonのプラグインシステムを初期化
    Python::attach(|py| -> PyResult<()> {
        // pythonのversionを取得
        let sys = py.import("sys")?;
        let version = sys.getattr("version_info")?;
        let major: i32 = version.get_item(0)?.extract()?;
        let minor: i32 = version.get_item(1)?.extract()?;
        // pathにvenvのsite-packagesを追加
        let site = py.import("site")?;
        let venv_site = venv_path
            .join("lib")
            .join(format!("python{}.{}", major, minor))
            .join("site-packages");
        site.call_method1("addsitedir", (venv_site.to_str().unwrap(),))?;
        println!("Added venv site-packages to PYTHONPATH");
        // sys.pathを表示
        let sys_path: Vec<String> = sys.getattr("path")?.extract()?;
        println!("sys.path: {:?}", sys_path);

        // plmanagerのPluginManagerを初期化
        let pl_manager = py.import("plmanager")?;
        let init_func = pl_manager.getattr("PluginManager")?;
        init_func.call1((&appdata_dir,))?;
        Ok(())
    })?;

    Ok(())
}
