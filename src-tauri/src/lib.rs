use pyo3::{types::PyAnyMethods, Python};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    Python::attach(|py| -> Result<(), pyo3::PyErr> {
        println!("\n--- RustからPythonの呼び出しを開始 ---");

        let sys = py.import("sys")?;
        let version: String = sys.getattr("version")?.extract()?;
        println!("Pythonのバージョン: {}", version);

        let os = py.import("os")?;
        let user: String = os.getattr("getlogin")?.call0()?.extract()?;
        println!("現在のユーザー: {}", user);

        let result: i64 = py.eval(c"10 + 20 * 5", None, None)?.extract()?;
        println!("Pythonで計算した結果 (10 + 20 * 5): {}", result);

        println!("--- Pythonの呼び出しが完了 ---");

        Ok(())
    }).unwrap();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
