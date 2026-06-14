//! Seedream 壁纸生成模块 (AI-002)
//!
//! 调用火山引擎 Seedream API 进行两步管线中的第二步：专业壁纸生成。
//! 支持 1/3/5 张变体，分辨率：1080P / 2K / 4K。
//! 生成完成后将图片下载到壁纸库目录。

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const VOLCANO_BASE_URL: &str = "https://ark.cn-beijing.volces.com/api/v3";
const SEEDREAM_MODEL: &str = "Seedream 4.0";
const SEEDREAM_PREMIUM_MODEL: &str = "Seedream 5.0 lite";
const STEP2_TIMEOUT_SECS: u64 = 30; // 图片生成可能较慢

/// 第二步生成请求参数（前端传入）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateRequest {
    /// 优化后的 prompt
    pub prompt: String,
    /// 模型名称
    pub model: Option<String>,
    /// 生成数量（1/3/5）
    pub count: Option<u32>,
    /// 分辨率
    pub resolution: Option<String>,
    /// 风格标签（拼入 prompt 增强）
    pub style_tags: Option<Vec<String>>,
    /// 色调
    pub color_scheme: Option<String>,
    /// 构图
    pub composition: Option<String>,
}

/// 单张生成结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedImage {
    /// 本地文件路径
    pub path: String,
    /// 文件名
    pub filename: String,
    /// 图片宽度
    pub width: u32,
    /// 图片高度
    pub height: u32,
    /// 文件大小（字节）
    pub file_size: u64,
}

/// 生成结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateResult {
    /// 生成的图片列表
    pub images: Vec<GeneratedImage>,
    /// 实际使用的模型
    pub model: String,
    /// 实际分辨率
    pub resolution: String,
}

// ---- Seedream API 请求/响应结构 ----

#[derive(Debug, Serialize)]
struct SeedreamRequest {
    model: String,
    prompt: String,
    n: u32,
    size: String,
    #[serde(rename = "response_format")]
    response_format: String,
}

#[derive(Debug, Deserialize)]
struct SeedreamResponse {
    data: Vec<SeedreamImageData>,
}

#[derive(Debug, Deserialize)]
struct SeedreamImageData {
    /// URL of the generated image
    url: Option<String>,
    /// Base64 encoded image data
    b64_json: Option<String>,
}

/// 解析分辨率字符串为 Seedream API 格式
fn parse_resolution(resolution: &str) -> (u32, u32) {
    match resolution {
        "2K" => (2560, 1440),
        "4K" => (3840, 2160),
        _ => (1920, 1080), // 默认 1080P
    }
}

/// 构建增强后的 prompt（融合风格、色调、构图信息）
fn build_enhanced_prompt(request: &GenerateRequest) -> String {
    let mut parts = vec![request.prompt.clone()];

    if let Some(ref tags) = request.style_tags {
        if !tags.is_empty() {
            parts.push(format!("Style: {}", tags.join(", ")));
        }
    }
    if let Some(ref color) = request.color_scheme {
        if !color.is_empty() {
            parts.push(format!("Color scheme: {}", color));
        }
    }
    if let Some(ref comp) = request.composition {
        if !comp.is_empty() {
            parts.push(format!("Composition: {}", comp));
        }
    }

    parts.join(". ")
}

/// 壁纸生成 (AI-002)
///
/// 调用 Seedream API 生成壁纸图片，下载到壁纸库目录。
#[tauri::command]
pub async fn ai_generate(
    app: tauri::AppHandle,
    api_key: String,
    request: GenerateRequest,
) -> Result<GenerateResult, String> {
    if api_key.is_empty() {
        return Err("API Key 未配置".to_string());
    }
    if request.prompt.trim().is_empty() {
        return Err("Prompt 不能为空".to_string());
    }

    let model = request
        .model
        .clone()
        .unwrap_or_else(|| SEEDREAM_MODEL.to_string());
    let count = request.count.unwrap_or(3).clamp(1, 5);
    let resolution_str = request
        .resolution
        .clone()
        .unwrap_or_else(|| "1080P".to_string());
    let (width, height) = parse_resolution(&resolution_str);
    let size = format!("{}x{}", width, height);
    let enhanced_prompt = build_enhanced_prompt(&request);

    let seedream_request = SeedreamRequest {
        model: model.clone(),
        prompt: enhanced_prompt,
        n: count,
        size,
        response_format: "b64_json".to_string(), // 直接获取 base64，避免外网 URL 不可达
    };

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(STEP2_TIMEOUT_SECS))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

    let url = format!("{}/images/generations", VOLCANO_BASE_URL);

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&seedream_request)
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                "生成超时，请重试".to_string()
            } else if e.is_connect() {
                "无法连接到火山引擎，请检查网络".to_string()
            } else {
                format!("网络错误: {}", e)
            }
        })?;

    let status = response.status().as_u16();
    if status != 200 {
        let body = response.text().await.unwrap_or_default();
        return Err(match status {
            401 => "API Key 无效".to_string(),
            403 => "API Key 权限不足".to_string(),
            429 => "请求过于频繁，请稍后再试".to_string(),
            400 => {
                if body.contains("content") || body.contains("safety") {
                    "生成内容未通过安全审核，请修改描述后重试".to_string()
                } else {
                    format!("请求参数错误: {}", truncate(&body, 200))
                }
            }
            402 | 422 => "API 余额不足，请前往火山引擎控制台充值".to_string(),
            _ => format!("服务器返回 HTTP {}: {}", status, truncate(&body, 200)),
        });
    }

    let seedream_response: SeedreamResponse = response
        .json()
        .await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    if seedream_response.data.is_empty() {
        return Err("AI 未返回任何图片".to_string());
    }

    // 下载/保存图片到壁纸库目录
    let wallpaper_dir = get_wallpaper_dir(&app);
    std::fs::create_dir_all(&wallpaper_dir)
        .map_err(|e| format!("创建壁纸目录失败: {}", e))?;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let mut images = Vec::new();
    for (i, img_data) in seedream_response.data.iter().enumerate() {
        let image_bytes = if let Some(ref b64) = img_data.b64_json {
            base64_decode(b64)?
        } else if let Some(ref url) = img_data.url {
            // 降级：从 URL 下载
            download_image(&client, url).await?
        } else {
            continue;
        };

        let filename = format!("ai_{}_{}_{}.png", timestamp, model_slug(&model), i + 1);
        let path = wallpaper_dir.join(&filename);

        let file_size = image_bytes.len() as u64;
        std::fs::write(&path, &image_bytes)
            .map_err(|e| format!("保存图片失败: {}", e))?;

        images.push(GeneratedImage {
            path: path.to_string_lossy().to_string(),
            filename,
            width,
            height,
            file_size,
        });
    }

    if images.is_empty() {
        return Err("未能保存任何生成的图片".to_string());
    }

    Ok(GenerateResult {
        images,
        model,
        resolution: resolution_str,
    })
}

/// 从 URL 下载图片（降级路径）
async fn download_image(client: &reqwest::Client, url: &str) -> Result<Vec<u8>, String> {
    let response = client
        .get(url)
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await
        .map_err(|e| format!("下载图片失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("下载图片 HTTP {}", response.status()));
    }

    response
        .bytes()
        .await
        .map(|b| b.to_vec())
        .map_err(|e| format!("读取图片数据失败: {}", e))
}

/// 获取壁纸库目录
fn get_wallpaper_dir(app: &tauri::AppHandle) -> PathBuf {
    use tauri::Manager;
    app.path()
        .app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("wallpapers")
}

/// 模型名转文件安全 slug
fn model_slug(model: &str) -> String {
    model
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' { c } else { '_' })
        .collect::<String>()
        .to_lowercase()
}

/// 简单 Base64 解码
fn base64_decode(s: &str) -> Result<Vec<u8>, String> {
    const DECODE: [i8; 128] = {
        let mut table = [-1i8; 128];
        let chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut i = 0;
        while i < chars.len() {
            table[chars[i] as usize] = i as i8;
            i += 1;
        }
        table
    };

    let bytes = s.as_bytes();
    let mut result = Vec::with_capacity(bytes.len() * 3 / 4);
    let mut buffer = 0u32;
    let mut bits = 0u32;

    for &b in bytes {
        let val = DECODE.get(b as usize).copied().unwrap_or(-1);
        if val < 0 {
            continue;
        }
        buffer = (buffer << 6) | val as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            result.push((buffer >> bits) as u8);
            buffer &= (1 << bits) - 1;
        }
    }

    Ok(result)
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_resolution_1080p() {
        assert_eq!(parse_resolution("1080P"), (1920, 1080));
    }

    #[test]
    fn test_parse_resolution_2k() {
        assert_eq!(parse_resolution("2K"), (2560, 1440));
    }

    #[test]
    fn test_parse_resolution_4k() {
        assert_eq!(parse_resolution("4K"), (3840, 2160));
    }

    #[test]
    fn test_build_enhanced_prompt_full() {
        let req = GenerateRequest {
            prompt: "cyberpunk city".into(),
            model: None,
            count: None,
            resolution: None,
            style_tags: Some(vec!["cyberpunk".into(), "neon".into()]),
            color_scheme: Some("purple+blue".into()),
            composition: Some("low angle".into()),
        };
        let result = build_enhanced_prompt(&req);
        assert!(result.contains("Style: cyberpunk, neon"));
        assert!(result.contains("Color scheme: purple+blue"));
        assert!(result.contains("Composition: low angle"));
    }

    #[test]
    fn test_model_slug() {
        assert_eq!(model_slug("Seedream 4.0"), "seedream_4_0");
        assert_eq!(model_slug("doubao-2.0-vision"), "doubao-2_0-vision");
    }

    #[test]
    fn test_base64_decode() {
        // "hello" in base64
        let decoded = base64_decode("aGVsbG8").unwrap();
        assert_eq!(decoded, b"hello");
    }
}
