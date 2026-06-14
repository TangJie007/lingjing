//! 豆包 VL-LLM 意图分析模块 (AI-001)
//!
//! 调用火山引擎 doubao API 进行两步管线中的第一步：意图分析与方案构建。
//! - 纯文字输入 → doubao-2.0-lite-32k
//! - 文字 + 参考图 → doubao-2.0-vision
//!
//! 输出结构化构建方案 JSON：optimized_prompt, style_tags, color_scheme, composition

use serde::{Deserialize, Serialize};

const VOLCANO_BASE_URL: &str = "https://ark.cn-beijing.volces.com/api/v3";
const DOUBAO_VISION_MODEL: &str = "doubao-2.0-vision";
const DOUBAO_LITE_MODEL: &str = "doubao-2.0-lite-32k";
const STEP1_TIMEOUT_SECS: u64 = 8; // 略宽松于 3s AC 要求，给网络留余量

/// 第一步分析请求参数（前端传入）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeRequest {
    /// 用户中文描述
    pub prompt: String,
    /// 可选参考图 Base64（不含 data:xxx;base64, 前缀）
    pub reference_image: Option<String>,
    /// 参考图的 MIME 类型，如 "image/png"
    pub image_mime: Option<String>,
}

/// 构建方案（AI 返回 + 前端展示）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildPlan {
    /// 优化后的英文 prompt
    pub optimized_prompt: String,
    /// 风格标签列表
    pub style_tags: Vec<String>,
    /// 色调方案
    pub color_scheme: String,
    /// 构图方向
    pub composition: String,
}

/// OpenAI-compatible 消息格式
#[derive(Debug, Clone, Serialize)]
struct ChatMessage {
    role: String,
    content: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    max_tokens: u32,
    #[serde(rename = "response_format")]
    response_format: ResponseFormat,
}

#[derive(Debug, Clone, Serialize)]
struct ResponseFormat {
    #[serde(rename = "type")]
    format_type: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatResponseMessage,
}

#[derive(Debug, Deserialize)]
struct ChatResponseMessage {
    content: String,
}

/// 系统提示词：指导 doubao 输出结构化构建方案
const SYSTEM_PROMPT: &str = r#"你是一个专业的动态桌面壁纸设计助手。用户会用中文描述想要的壁纸效果，你需要分析并输出一个结构化的壁纸构建方案。

请严格按以下 JSON 格式输出（不要包含 markdown 代码块标记，只输出纯 JSON）：

{
  "optimized_prompt": "优化后的英文 prompt，用于图片生成模型。必须详细描述画面内容、风格、光影、色彩、构图，80-200 词",
  "style_tags": ["风格标签1", "风格标签2", "风格标签3"],
  "color_scheme": "主色调和配色方案的简短描述，如'赛博朋克霓虹紫+深蓝'",
  "composition": "构图方向的简短描述，如'低角度仰拍、中心对称、引导线汇聚'"
}

要求：
1. optimized_prompt 必须是英文，适合直接输入 Seedream 等图片生成模型
2. style_tags 提供 2-4 个准确的风格标签
3. 如果有参考图，结合参考图的视觉特征进行分析
4. 壁纸应适合 16:9 桌面比例"#;

/// AI 意图分析与方案构建 (AI-001)
///
/// 调用 doubao VL-LLM 分析用户描述，输出结构化构建方案。
/// 3 秒内返回（AC 要求）。
#[tauri::command]
pub async fn ai_analyze(
    api_key: String,
    request: AnalyzeRequest,
) -> Result<BuildPlan, String> {
    if api_key.is_empty() {
        return Err("API Key 未配置".to_string());
    }
    if request.prompt.trim().is_empty() {
        return Err("描述不能为空".to_string());
    }

    // 选择模型：有参考图用 vision，纯文字用 lite
    let model = if request.reference_image.is_some() {
        DOUBAO_VISION_MODEL
    } else {
        DOUBAO_LITE_MODEL
    };

    // 构建消息内容
    let user_content = build_user_content(&request)?;

    let chat_request = ChatRequest {
        model: model.to_string(),
        messages: vec![
            ChatMessage {
                role: "system".to_string(),
                content: serde_json::Value::String(SYSTEM_PROMPT.to_string()),
            },
            ChatMessage {
                role: "user".to_string(),
                content: user_content,
            },
        ],
        temperature: 0.7,
        max_tokens: 1024,
        response_format: ResponseFormat {
            format_type: "json_object".to_string(),
        },
    };

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(STEP1_TIMEOUT_SECS))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

    let url = format!("{}/chat/completions", VOLCANO_BASE_URL);

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&chat_request)
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                "分析超时，请检查网络后重试".to_string()
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
            401 => "API Key 无效，请前往设置页重新配置".to_string(),
            403 => "API Key 权限不足".to_string(),
            429 => "请求过于频繁，请稍后再试".to_string(),
            _ => format!("服务器返回 HTTP {}: {}", status, truncate(&body, 200)),
        });
    }

    let chat_response: ChatResponse = response
        .json()
        .await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    let content = chat_response
        .choices
        .first()
        .ok_or("AI 未返回结果")?
        .message
        .content
        .clone();

    // 解析 JSON 响应（处理可能的 markdown 代码块包裹）
    let json_str = extract_json(&content);
    let plan: BuildPlan =
        serde_json::from_str(json_str).map_err(|e| format!("解析构建方案失败: {}", e))?;

    if plan.optimized_prompt.is_empty() {
        return Err("AI 返回的 prompt 为空，请重试".to_string());
    }

    Ok(plan)
}

/// 构建用户消息内容（支持纯文字和多模态）
fn build_user_content(request: &AnalyzeRequest) -> Result<serde_json::Value, String> {
    if let Some(ref image_b64) = request.reference_image {
        // 多模态：文字 + 图片
        let mime = request.image_mime.as_deref().unwrap_or("image/png");
        Ok(serde_json::json!([
            {
                "type": "text",
                "text": format!("请分析以下壁纸描述和参考图，输出构建方案：\n\n描述：{}", request.prompt)
            },
            {
                "type": "image_url",
                "image_url": {
                    "url": format!("data:{};base64,{}", mime, image_b64)
                }
            }
        ]))
    } else {
        // 纯文字
        Ok(serde_json::json!([
            {
                "type": "text",
                "text": format!("请分析以下壁纸描述，输出构建方案：\n\n描述：{}", request.prompt)
            }
        ]))
    }
}

/// 从 AI 响应中提取 JSON（处理可能的 markdown 代码块包裹）
fn extract_json(content: &str) -> &str {
    let content = content.trim();
    // 尝试去掉 ```json ... ``` 包裹
    if content.starts_with("```json") {
        if let Some(end) = content.rfind("```") {
            let start = "```json".len();
            return content[start..end].trim();
        }
    }
    if content.starts_with("```") {
        if let Some(end) = content.rfind("```") {
            let start = "```".len();
            return content[start..end].trim();
        }
    }
    content
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
    fn test_extract_json_plain() {
        let input = r#"{"optimized_prompt": "test", "style_tags": [], "color_scheme": "blue", "composition": "center"}"#;
        let result = extract_json(input);
        assert!(result.starts_with("{"));
    }

    #[test]
    fn test_extract_json_code_block() {
        let input = "```json\n{\"optimized_prompt\": \"test\"}\n```";
        let result = extract_json(input);
        assert_eq!(result, "{\"optimized_prompt\": \"test\"}");
    }

    #[test]
    fn test_extract_json_code_block_no_lang() {
        let input = "```\n{\"test\": true}\n```";
        let result = extract_json(input);
        assert_eq!(result, "{\"test\": true}");
    }
}
