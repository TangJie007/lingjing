//! Thumbnail generation for wallpaper library (WL-002)
//! Video: MPV frame capture → WebP. GIF: first frame via image crate.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Generate a thumbnail for a wallpaper file.
/// Returns the path to the generated .thumb.webp file.
#[tauri::command]
pub fn generate_thumbnail(wallpaper_path: String, media_type: String) -> Result<String, String> {
    generate_thumbnail_internal(&wallpaper_path, &media_type)
}

/// Internal thumbnail generation (also used by wallpaper_engine on import).
pub fn generate_thumbnail_internal(
    wallpaper_path: &str,
    media_type: &str,
) -> Result<String, String> {
    let path = PathBuf::from(wallpaper_path);
    let thumb_path = thumb_path_for(&path);
    if thumb_path.exists() {
        return Ok(thumb_path.to_string_lossy().to_string());
    }

    match media_type {
        "video" => generate_video_thumbnail(&path, &thumb_path),
        "gif" => generate_gif_thumbnail(&path, &thumb_path),
        _ => Ok(wallpaper_path.to_string()),
    }
}

/// Compute the thumbnail path for a wallpaper file: `{name}.thumb.webp`
pub fn thumb_path_for(wallpaper_path: &Path) -> PathBuf {
    let mut thumb = wallpaper_path.to_path_buf();
    thumb.set_extension("thumb.webp");
    thumb
}

/// Generate thumbnail for video files using bundled MPV.
fn generate_video_thumbnail(source: &Path, dest: &Path) -> Result<String, String> {
    let mpv = find_mpv().ok_or("未找到 mpv.exe。请运行 pnpm setup:mpv 或安装系统 MPV")?;

    let temp_png = dest.with_extension("tmp.png");
    let _ = std::fs::remove_file(&temp_png);

    let output = Command::new(&mpv)
        .args([
            "--no-config",
            "--no-terminal",
            "--really-quiet",
            "--no-audio",
            "--start=0.5",
            "--frames=1",
            &format!("--o={}", temp_png.to_string_lossy()),
            &source.to_string_lossy(),
        ])
        .output()
        .map_err(|e| format!("MPV 执行失败: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let _ = std::fs::remove_file(&temp_png);
        return Err(format!("MPV 缩略图生成失败: {}", stderr.trim()));
    }

    if !temp_png.exists() {
        return Err("MPV 未输出缩略图文件".into());
    }

    encode_png_as_webp(&temp_png, dest)?;
    let _ = std::fs::remove_file(&temp_png);

    log::info!("视频缩略图已生成: {}", dest.display());
    Ok(dest.to_string_lossy().to_string())
}

/// Generate thumbnail for GIF files using the image crate.
fn generate_gif_thumbnail(source: &Path, dest: &Path) -> Result<String, String> {
    use image::codecs::gif::GifDecoder;
    use image::AnimationDecoder;
    use std::io::BufReader;

    let file = std::fs::File::open(source)
        .map_err(|e| format!("无法打开 GIF 文件: {}", e))?;

    let reader = BufReader::new(file);
    let decoder = GifDecoder::new(reader)
        .map_err(|e| format!("GIF 解码失败: {}", e))?;

    let frames = decoder.into_frames();
    let first_frame = frames
        .into_iter()
        .next()
        .ok_or("GIF 无帧")?
        .map_err(|e| format!("GIF 帧读取失败: {}", e))?;

    let frame_buffer = first_frame.into_buffer();
    let dynamic_image = image::DynamicImage::ImageRgba8(frame_buffer);

    let encoder = webp::Encoder::from_image(&dynamic_image)
        .map_err(|e| format!("WebP 编码失败: {}", e))?;

    let webp_data = encoder.encode(80.0);

    std::fs::write(dest, &*webp_data)
        .map_err(|e| format!("WebP 写入失败: {}", e))?;

    log::info!("GIF 缩略图已生成: {}", dest.display());
    Ok(dest.to_string_lossy().to_string())
}

fn encode_png_as_webp(source: &Path, dest: &Path) -> Result<(), String> {
    let img = image::open(source).map_err(|e| format!("读取缩略图失败: {}", e))?;
    let encoder = webp::Encoder::from_image(&img).map_err(|e| format!("WebP 编码失败: {}", e))?;
    let webp_data = encoder.encode(80.0);
    std::fs::write(dest, &*webp_data).map_err(|e| format!("WebP 写入失败: {}", e))?;
    Ok(())
}

fn find_mpv() -> Option<PathBuf> {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            for candidate in [
                dir.join("bin").join("mpv").join("mpv.exe"),
                dir.join("mpv.exe"),
            ] {
                if candidate.exists() {
                    return Some(candidate);
                }
            }
        }
    }

    std::env::var_os("PATH").and_then(|path_var| {
        std::env::split_paths(&path_var)
            .map(|d| d.join("mpv.exe"))
            .find(|p| p.exists())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thumb_path_for() {
        let p = Path::new("C:/test/wallpaper.mp4");
        let thumb = thumb_path_for(p);
        assert_eq!(
            thumb.to_string_lossy(),
            "C:/test/wallpaper.thumb.webp"
        );
    }
}
