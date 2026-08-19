fn main() {
    // 图标在编译期嵌入 exe；变更后必须触发 build.rs 重跑
    println!("cargo:rerun-if-changed=tauri.conf.json");
    if let Ok(entries) = std::fs::read_dir("icons") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                println!("cargo:rerun-if-changed={}", path.display());
            }
        }
    }
    tauri_build::build()
}
