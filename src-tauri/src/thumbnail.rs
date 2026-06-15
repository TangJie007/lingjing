//! Thumbnail generation for wallpaper library (WL-002)
//! Generates WebP thumbnails for video (MP4/WebM via FFmpeg) and GIF (via image crate).
//! Runs in background threads to avoid blocking the UI.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Generate a thumbnail for a wallpaper file.
/// Returns the path to the generated .thumb.webp file.
#[tauri::command]
pub fn generate_thumbnail(wallpaper_path: String, media_type: String) -> Result<String, String> {
    let path = PathBuf::from(&wallpaper_path);
    let thumb_path = thumbnail_path(&path);
    if thumb_path.exists() {
        return Ok(thumb_path.to_string_lossy().to_string());
    }

    match media_type.as_str() {
        "video" => generate_video_thumbnail(&path, &thumb_path),
        "gif" => generate_gif_thumbnail(&path, &thumb_path),
        _ => {
            // For static images, just return the original path (no thumbnail needed)
            Ok(wallpaper_path)
        }
    }
}

/// Generate thumbnail for video files using FFmpeg.
/// Extracts frame at 0.5 seconds, encodes as WebP.
fn generate_video_thumbnail(source: &Path, dest: &Path) -> Result<String, String> {
    let ffmpeg = find_ffmpeg().ok_or("未找到 ffmpeg.exe。请安装 FFmpeg 或将 ffmpeg.exe 放到 PATH 中")?;

    let output = Command::new(&ffmpeg)
        .args([
            "-ss", "0.5",
            "-i", &source.to_string_lossy(),
            "-vframes", "1",
            "-q:v", "80",
            "-y",
            &dest.to_string_lossy(),
        ])
        .output()
        .map_err(|e| format!("FFmpeg 执行失败: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("FFmpeg 缩略图生成失败: {}", stderr));
    }

    log::info!("视频缩略图已生成: {}", dest.display());
    Ok(dest.to_string_lossy().to_string())
}

/// Generate thumbnail for GIF files using the image crate.
/// Extracts the first frame and encodes as WebP.
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

    // Encode as WebP
    let encoder = webp::Encoder::from_image(&dynamic_image)
        .map_err(|e| format!("WebP 编码失败: {}", e))?;

    let webp_data = encoder.encode(80.0);

    std::fs::write(dest, &*webp_data)
        .map_err(|e| format!("WebP 写入失败: {}", e))?;

    log::info!("GIF 缩略图已生成: {}", dest.display());
    Ok(dest.to_string_lossy().to_string())
}

/// Compute the thumbnail path for a wallpaper file.
/// Format: {wallpaper_path_without_ext}.thumb.webp
fn thumbnail_path(wallpaper_path: &Path) -> PathBuf {
    let mut thumb = wallpaper_path.to_path_buf();
    thumb.set_extension("thumb.webp");
    thumb
}

/// Find FFmpeg executable in PATH or bundled directory.
fn find_ffmpeg() -> Option<PathBuf> {
    // Check bundled first
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let bundled = dir.join("bin").join("ffmpeg").join("ffmpeg.exe");
            if bundled.exists() {
                return Some(bundled);
            }
            let bundled2 = dir.join("ffmpeg.exe");
            if bundled2.exists() {
                return Some(bundled2);
            }
        }
    }

    // Check PATH
    std::env::var_os("PATH").and_then(|path_var| {
        std::env::split_paths(&path_var)
            .map(|d| d.join("ffmpeg.exe"))
            .find(|p| p.exists())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thumbnail_path() {
        let p = Path::new("C:/test/wallpaper.mp4");
        let thumb = thumbnail_path(p);
        assert_eq!(
            thumb.to_string_lossy(),
            "C:/test/wallpaper.thumb.webp"
        );
    }
}
