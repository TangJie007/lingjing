## Why

灵境（LingScape）是一款 AI 动态桌面创作工具——用户用中文描述想要的桌面效果，AI 理解意图后调用专业模型生成高质量壁纸。V1.0 目标是 2026 Q4 上线 Steam，¥9.9 买断制，纯 BYOK（用户自带 API Key）零服务器架构。

本设计覆盖全部 21 个 Spec 功能点（7 模块），技术选型为 **Tauri + Vue 3 + Vite**，面向 Windows 平台首发。

## What Changes

- **技术栈确立**：Tauri (Rust) 作为桌面壳 → Vue 3 + Vite 作为 UI 层，替代此前待定的 Electron 方案
- **新增国际化框架（SET-005）**：所有 UI 文案统一走 i18n key，本期实现 zh-CN，预留 en 骨架
- **首次完整设计**：21 个功能点的模块拆分、数据流、文件布局、测试策略

## Capabilities

### New Capabilities

| 能力 | Spec | 说明 |
|------|------|------|
| Tauri 桌面壳 | 全局 | Rust 原生窗口管理、系统托盘、全屏检测、开机自启 |
| Vue 3 SPA 前端 | 全局 | 所有 UI 页面和交互逻辑 |
| Vite 构建 | 全局 | 开发热更新 + 生产打包 |
| 两步 AI 管线 | AI-001 ~ AI-003 | VL-LLM 意图分析 → Seedream 壁纸生成 |
| API Key 管理 | API-001 ~ API-003 | 加密存储 + 有效性检测 + 多平台卡片 |
| 壁纸播放引擎 | WP-001 ~ WP-003 | 静态/视频壁纸 + 全屏检测暂停 |
| 壁纸库管理 | WL-001 ~ WL-002 | 本地壁纸 CRUD + AI 生成历史 |
| 系统托盘 | ST-001 | 最小化 + 右键菜单 |
| 首次启动向导 | OB-001 ~ OB-004 | 欢迎页 → API 引导 → 快速上手 → 本地模式 |
| 基础设置 | SET-001 ~ SET-005 | 通用/API/壁纸/性能/存储/i18n |
| 国际化框架 | SET-005 | i18n key 体系 + zh-CN 完整 + en 骨架 |

## Impact

- **Spec**：`specs/` 下 7 模块 21 功能，全部纳入本设计
- **配置**：新增 `src/config/` 统一管理常量（API 端点、模型参数、性能阈值等）
- **测试**：`src/<module-slug>/__tests__/`，按 AC 嵌套 describe
- **风险**：Tauri 在 Windows 壁纸渲染方面需验证可行性（通过 Rust Win32 API 或第三方 crate）
