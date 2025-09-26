use std::path::PathBuf;

use tauri::{AppHandle, Manager};

pub fn get_data_dir(app: &AppHandle) -> PathBuf {
    let appdata_dir = app.path().app_data_dir().unwrap();
    if !appdata_dir.exists() {
        println!("Creating app data directory at {:?}", &appdata_dir);
        std::fs::create_dir_all(&appdata_dir).unwrap();
    }
    appdata_dir
}

pub fn get_local_data_dir(app: &AppHandle) -> PathBuf {
    let local_data_dir = app.path().local_data_dir().unwrap();
    if !local_data_dir.exists() {
        println!("Creating local data directory at {:?}", &local_data_dir);
        std::fs::create_dir_all(&local_data_dir).unwrap();
    }
    local_data_dir
}
