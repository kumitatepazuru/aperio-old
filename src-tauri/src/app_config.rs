use std::{fs::File, io::Write};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Manager};

use crate::dir_util::get_data_dir;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PythonStruct {
    pub version: String,
    pub next_version: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub python: PythonStruct,
}

pub fn init_config(app: &AppHandle) {
    // 設定ファイルがあるか確認し、なければdefault-config.jsonをコピー
    let appdata_dir = get_data_dir(app);
    let config_path = appdata_dir.join("config.json");
    if !config_path.exists() {
        let config_bytes = include_bytes!("../data/default-config.json");
        let mut file = File::create(&config_path).unwrap();
        file.write(config_bytes).unwrap();
        file.sync_data().unwrap();
        println!("Default config copied to {:?}", config_path);
    } else {
        println!("Config file found at {:?}", config_path);
    }
}

pub fn read_config(app: &AppHandle) -> Option<AppConfig> {
    let appdata_dir = get_data_dir(app);
    let config_path = appdata_dir.join("config.json");
    if config_path.exists() {
        let config = std::fs::read_to_string(&config_path).ok()?;
        let config: AppConfig = match serde_json::from_str(&config) {
            Ok(config) => config,
            Err(err) => {
                println!(
                    "Failed to parse config.json: {}. Trying to merge with default config.",
                    err
                );
                let merged_config = merge_configs(&config);
                let config: AppConfig = serde_json::from_value(merged_config)
                    .expect("Failed to parse config even after merging with default config.");
                
                // 成功したなら保存
                let config_str = serde_json::to_string_pretty(&config).unwrap();
                std::fs::write(&config_path, config_str).unwrap();

                config
            }
        };
        Some(config)
    } else {
        None
    }
}

fn merge_configs(config: &str) -> Value {
    let default_config_bytes = include_bytes!("../data/default-config.json");
    let default_config: Value = serde_json::from_slice(default_config_bytes).unwrap();
    let user_config: Value = serde_json::from_str(config).unwrap();
    let mut merged_config = default_config;
    merge(&mut merged_config, user_config);

    merged_config
}

fn merge(a: &mut Value, b: Value) {
    if let Value::Object(a) = a {
        if let Value::Object(b) = b {
            for (k, v) in b {
                if v.is_null() {
                    a.remove(&k);
                } else {
                    merge(a.entry(k).or_insert(Value::Null), v);
                }
            }

            return;
        }
    }

    *a = b;
}
