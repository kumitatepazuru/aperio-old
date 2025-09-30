use crate::dir_util::get_data_dir;
use anyhow::Result;
use pyo3::prelude::PyAnyMethods;
use pyo3::{PyResult, Python};
use tauri::AppHandle;

pub fn initialize_python(app: AppHandle) -> Result<()> {
    let appdata_dir = get_data_dir(&app)?;

    // Pythonのプラグインシステムを初期化
    Python::attach(|py| -> PyResult<()> {
        // pythonのversionを取得
        let sys = py.import("sys")?;

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
