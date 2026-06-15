//! Update checker (SET-009)
//! Checks for new versions via GitHub Release API or custom JSON endpoint.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub has_update: bool,
    pub current_version: String,
    pub latest_version: String,
    pub download_url: Option<String>,
    pub release_notes: Option<String>,
}

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[tauri::command]
pub async fn check_update() -> Result<UpdateInfo, String> {
    // Try GitHub Releases API
    let url = "https://api.github.com/repos/linggou-tech/lingscape/releases/latest";

    let client = reqwest::Client::builder()
        .user_agent("LingScape-UpdateChecker/1.0")
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    match client.get(url).send().await {
        Ok(resp) => {
            if resp.status().is_success() {
                let json: serde_json::Value = resp
                    .json()
                    .await
                    .map_err(|e| format!("解析更新响应失败: {}", e))?;

                let latest = json["tag_name"]
                    .as_str()
                    .unwrap_or("0.0.0")
                    .trim_start_matches('v');

                let has_update = compare_versions(latest, CURRENT_VERSION);

                Ok(UpdateInfo {
                    has_update,
                    current_version: CURRENT_VERSION.into(),
                    latest_version: latest.into(),
                    download_url: json["html_url"].as_str().map(|s| s.to_string()),
                    release_notes: json["body"].as_str().map(|s| s.to_string()),
                })
            } else {
                // GitHub API not reachable, return no update
                Ok(UpdateInfo {
                    has_update: false,
                    current_version: CURRENT_VERSION.into(),
                    latest_version: CURRENT_VERSION.into(),
                    download_url: None,
                    release_notes: None,
                })
            }
        }
        Err(_) => Ok(UpdateInfo {
            has_update: false,
            current_version: CURRENT_VERSION.into(),
            latest_version: CURRENT_VERSION.into(),
            download_url: None,
            release_notes: None,
        }),
    }
}

fn compare_versions(latest: &str, current: &str) -> bool {
    let parse = |v: &str| -> Vec<u32> {
        v.split('.')
            .filter_map(|s| s.parse::<u32>().ok())
            .collect()
    };
    parse(latest) > parse(current)
}
