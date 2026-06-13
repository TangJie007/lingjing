fn main() {
    copy_bundled_mpv();
    tauri_build::build()
}

/// 将 src-tauri/bin/mpv 同步到 target/{profile}/bin/mpv，供运行时 resolve_mpv_executable 查找
fn copy_bundled_mpv() {
    use std::env;
    use std::fs;
    use std::path::PathBuf;

    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let src = manifest.join("bin").join("mpv").join("mpv.exe");
    if !src.exists() {
        println!(
            "cargo:warning=未找到 {}，视频壁纸需运行 pnpm setup:mpv 或安装系统 MPV",
            src.display()
        );
        return;
    }

    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".into());
    let dest_dir = manifest.join("target").join(&profile).join("bin").join("mpv");
    if let Err(e) = fs::create_dir_all(&dest_dir) {
        println!("cargo:warning=创建 MPV 目标目录失败: {e}");
        return;
    }

    let src_dir = manifest.join("bin").join("mpv");
    if let Err(e) = copy_dir_all(&src_dir, &dest_dir) {
        println!("cargo:warning=同步 MPV 失败: {e}");
        return;
    }

    println!("cargo:rerun-if-changed={}", src_dir.display());
}

fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    use std::fs;
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&from, &to)?;
        } else {
            fs::copy(&from, &to)?;
        }
    }
    Ok(())
}
