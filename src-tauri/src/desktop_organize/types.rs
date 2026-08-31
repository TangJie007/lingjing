use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopItem {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub kind: String,
    #[serde(default)]
    pub builtin: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShellMenuEntry {
    pub id: u32,
    pub label: String,
    pub disabled: bool,
    pub separator: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<ShellMenuEntry>>,
    pub menu_path: Vec<u32>,
    /// Win11-style pinned action in the top icon strip.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub pin: bool,
}
