//! 火山引擎 API 通信模块
//!
//! 负责：
//! - API Key 有效性检测 (API-002)：发送最小请求到火山引擎 `/models` 端点
//! - AI 意图分析 (AI-001)：doubao VL-LLM 结构化方案构建
//! - AI 壁纸生成 (AI-002)：Seedream 图片生成 + 本地下载
//! - `open_url` 命令：使用系统默认浏览器打开链接

pub mod doubao;
pub mod seedream;

use serde::{Deserialize, Serialize};

/// 火山引擎 API 基础 URL
const VOLCANO_BASE_URL: &str = "https://ark.cn-beijing.volces.com/api/v3";

/// 连接测试结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConnectionResult {
    /// 是否连接成功
    pub success: bool,
    /// 错误信息（失败时）
    pub error: Option<String>,
    /// HTTP 状态码
    pub status_code: Option<u16>,
}

/// 测试 API Key 有效性
///
/// 向火山引擎 `GET /models` 发送最小请求，仅验证 Key 是否有效。
/// 超时时间：5 秒（满足 AC 3 秒内返回的要求）。
#[tauri::command]
pub async fn test_connection(api_key: String) -> Result<TestConnectionResult, String> {
    if api_key.is_empty() {
        return Ok(TestConnectionResult {
            success: false,
            error: Some("API Key is empty".to_string()),
            status_code: None,
        });
    }

    let url = format!("{}/models", VOLCANO_BASE_URL);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

    match client
        .get(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status().as_u16();
            if status == 200 {
                Ok(TestConnectionResult {
                    success: true,
                    error: None,
                    status_code: Some(status),
                })
            } else {
                let body = response.text().await.unwrap_or_default();
                let error_msg = match status {
                    401 => "API Key 无效，请检查是否正确".to_string(),
                    403 => "API Key 权限不足".to_string(),
                    429 => "请求过于频繁，请稍后再试".to_string(),
                    _ => format!("服务器返回 HTTP {}: {}", status, truncate(&body, 200)),
                };
                Ok(TestConnectionResult {
                    success: false,
                    error: Some(error_msg),
                    status_code: Some(status),
                })
            }
        }
        Err(e) => {
            let error_msg = if e.is_timeout() {
                "连接超时，请检查网络".to_string()
            } else if e.is_connect() {
                "无法连接到火山引擎，请检查网络".to_string()
            } else {
                format!("网络错误: {}", e)
            };
            Ok(TestConnectionResult {
                success: false,
                error: Some(error_msg),
                status_code: None,
            })
        }
    }
}

/// 使用系统默认浏览器打开 URL
///
/// 用于引导页的注册链接等。
#[tauri::command]
pub fn open_url(url: String) -> Result<(), String> {
    open::that(&url).map_err(|e| format!("Failed to open URL: {}", e))
}

/// 截断字符串到指定长度，超出部分用 "..." 替代
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
    fn test_truncate() {
        assert_eq!(truncate("hello", 10), "hello");
        assert_eq!(truncate("hello world", 5), "hello...");
    }
}
