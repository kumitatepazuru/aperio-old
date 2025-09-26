fn main() {
    // rpathを通す
    if cfg!(target_os = "linux") {
        // linuxの場合
        println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/../lib/aperio/,-rpath,/usr/lib/aperio/");
    } else if cfg!(target_os = "macos") {
        // macosの場合
        println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path/../Resources/");
    }

    tauri_build::build()
}
