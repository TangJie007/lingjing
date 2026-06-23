//! Auto-organization rules (DO-004)
//! Maps desktop files to a tidy-up category used by the one-click organizer.

use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoRule {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub file_types: Vec<String>,
    pub keywords: Vec<String>,
    pub target_category: String,
}

pub struct RuleState {
    pub rules: Mutex<Vec<AutoRule>>,
}

impl Default for RuleState {
    fn default() -> Self {
        Self {
            rules: Mutex::new(vec![
                AutoRule {
                    id: "rule_images".into(),
                    name: "图片文件 → 图片与视频".into(),
                    enabled: false,
                    file_types: vec![".jpg".into(), ".png".into(), ".gif".into(), ".webp".into()],
                    keywords: vec![],
                    target_category: "media".into(),
                },
                AutoRule {
                    id: "rule_docs".into(),
                    name: "文档文件 → 文档".into(),
                    enabled: false,
                    file_types: vec![".pdf".into(), ".docx".into(), ".txt".into(), ".md".into()],
                    keywords: vec![],
                    target_category: "docs".into(),
                },
                AutoRule {
                    id: "rule_screenshots".into(),
                    name: "截图文件 → 图片与视频".into(),
                    enabled: false,
                    file_types: vec![".png".into(), ".jpg".into()],
                    keywords: vec!["截图".into(), "screenshot".into()],
                    target_category: "media".into(),
                },
            ]),
        }
    }
}

#[tauri::command]
pub fn get_auto_rules(
    state: tauri::State<'_, RuleState>,
) -> Result<Vec<AutoRule>, String> {
    let guard = state.rules.lock().map_err(|e| e.to_string())?;
    Ok(guard.clone())
}

#[tauri::command]
pub fn update_auto_rule(
    state: tauri::State<'_, RuleState>,
    rule_id: String,
    enabled: Option<bool>,
    target_category: Option<String>,
) -> Result<(), String> {
    let mut guard = state.rules.lock().map_err(|e| e.to_string())?;
    if let Some(rule) = guard.iter_mut().find(|r| r.id == rule_id) {
        if let Some(v) = enabled { rule.enabled = v; }
        if let Some(v) = target_category { rule.target_category = v; }
    }
    Ok(())
}

#[tauri::command]
pub fn apply_auto_rules(
    state: tauri::State<'_, RuleState>,
) -> Result<Vec<String>, String> {
    let guard = state.rules.lock().map_err(|e| e.to_string())?;
    let active: Vec<_> = guard.iter().filter(|r| r.enabled).collect();
    Ok(active.iter().map(|r| r.name.clone()).collect())
}
