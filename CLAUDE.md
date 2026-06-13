# 灵境（LingScape）— AI 动态桌面创作工具

> 项目上下文文件，Claude Code 每次新对话自动加载。

## 产品定位

- **一句话**：用 AI 创造独一无二的动态桌面——你描述，AI 构建
- **定价**：¥9.9 买断制（Steam）
- **商业模式**：纯 BYOK（用户自带火山引擎 API Key），零后端
- **目标平台**：Windows 10 21H2+ / Windows 11
- **上线窗口**：2026 Q4 Steam

## 技术栈

| 层 | 技术 |
|----|------|
| 桌面框架 | **Tauri 2.x**（Rust 后端 + WebView2 前端） |
| 前端 | **Vue 3.5** + **Pinia** + **vue-router** (Hash 模式) + **vue-i18n** |
| 样式 | **Tailwind CSS v4** + 自定义设计令牌（"灵境流光"设计系统） |
| 构建 | **Vite 8** + **pnpm** |
| Rust 依赖 | `tauri 2.11`, `windows 0.58`, `serde/serde_json`, `tauri-plugin-autostart`, `tauri-plugin-log` |
| AI 模型 | 火山引擎：doubao-2.0-vision/lite（第一步） + Seedream 4.0（第二步） |
| 国际化 | vue-i18n (Composition API)，zh-CN 完整 + en 骨架 |

## 目录结构

```
ling-scape/
├── src/                          # Vue 前端
│   ├── App.vue                   # 根组件（主窗口 + 桌面播放器路由分发）
│   ├── main.ts                   # 入口：createApp → pinia → router → i18n
│   ├── assets/styles/main.css    # 全局样式 + 设计令牌 + 组件 CSS
│   ├── components/
│   │   ├── ai/AiCreatePanel.vue  # AI 创作底部面板
│   │   ├── common/Toast.vue      # Toast 通知
│   │   ├── common/ToggleSwitch.vue # 切换开关
│   │   ├── layout/               # AppTitlebar, AppSidebar, AppStatusbar, FabCreate, HazyBackground, SearchOverlay
│   │   └── wallpaper/WallpaperCard.vue # 壁纸卡片
│   ├── composables/
│   │   ├── useTray.ts            # 系统托盘事件监听
│   │   └── useWindow.ts          # 窗口关闭拦截（最小化到托盘）
│   ├── config/constants.ts       # 应用常量
│   ├── i18n/
│   │   ├── index.ts              # vue-i18n 初始化 + 格式化函数
│   │   └── locales/{zh-CN,en}.json
│   ├── router/index.ts           # Hash 路由
│   ├── stores/
│   │   ├── app.ts                # 全局状态（侧边栏、首次启动、locale）
│   │   ├── settings.ts           # 设置持久化（localStorage）
│   │   ├── api-keys.ts           # API Key 管理
│   │   └── wallpaper.ts          # 壁纸库状态管理
│   └── views/
│       ├── OnboardingPage.vue    # 首次向导（3步）
│       ├── LibraryPage.vue       # 壁纸库（网格、筛选、导入、CRUD）
│       ├── AiCreatePage.vue      # AI 创作页（占位）
│       ├── ApiKeysPage.vue       # API 管理（重定向到 Settings）
│       ├── SettingsPage.vue      # 设置页
│       └── DesktopPlayer.vue     # 桌面视频播放器（独立窗口渲染）
├── src-tauri/                    # Rust 后端
│   ├── Cargo.toml
│   ├── tauri.conf.json           # Tauri 配置（窗口、打包、安全策略）
│   └── src/
│       ├── main.rs               # Windows 入口
│       ├── lib.rs                # Tauri Builder + 命令注册
│       ├── tray.rs               # 系统托盘（右键菜单）
│       ├── autostart.rs          # 开机自启
│       └── wallpaper_engine.rs   # 壁纸引擎（Windows API、文件管理、全屏检测、桌面播放器窗口）
├── specs/                        # wetspec 功能规格（21个功能，7个模块）
├── docs/                         # 文档
│   └── 动态桌面实现原理.md        # 动态桌面架构文档
├── ui/                           # 设计系统文档
├── wetspec/                      # wetspec 变更记录
└── package.json                  # pnpm workspace
```

## 常用命令

```bash
pnpm dev              # 启动 Vite 开发服务器（前端热更新）
pnpm dev:tauri        # 启动 Tauri 开发模式（含 Rust 编译 + 前端）
pnpm build            # vue-tsc 类型检查 + Vite 生产构建
pnpm build:tauri      # Tauri 完整打包
cargo build --manifest-path src-tauri/Cargo.toml  # 仅编译 Rust
```

## 架构要点

### 双窗口架构（动态桌面核心）
- **主窗口**（label: `main`）：无边框窗口，自定义标题栏，壁纸库/AI创作/设置
- **桌面播放器窗口**（label: `desktop-player`）：程序化创建，`always_on_bottom`，全屏覆盖主显示器，用于渲染视频壁纸。创建逻辑在 `wallpaper_engine.rs:ensure_desktop_player_window()`
- 静态壁纸：调用 `SystemParametersInfoW(SPI_SETDESKWALLPAPER)` 设置 Windows 桌面
- 视频壁纸：显示桌面播放器窗口，通过 `<video>` 标签播放
- Rust↔Vue 通信：`app.emit("wallpaper-changed")` 事件 + `invoke()` 命令

### 设计系统
- 品牌色：翡翠青 `#008336`（primary）、琥珀金 `#aa6300`（accent）
- 渐变主色：`linear-gradient(135deg, #008336, #068d9a)`
- 字体：SmileySans（展示）、LXGW WenKai（正文）、Inter（UI）
- 圆角系统：sm(6) / md(10) / lg(14) / xl(20)

### 开发阶段

| 阶段 | 状态 | 内容 |
|------|:--:|------|
| 一：基础骨架 | ✅ 完成 | i18n、托盘、欢迎页、设置 UI |
| 二：壁纸引擎 | ✅ 完成 | 静态/视频壁纸、壁纸库管理、全屏检测 |
| 三：API 基础设施 | 🔜 待开发 | API Key 配置+检测、向导优化 |
| 四：AI 核心 | 🔜 待开发 | 两步 AI 管线、方案构建、壁纸生成 |
| 五：收尾 | 🔜 待开发 | 快速上手、多平台 API、完整闭环 |

## 关键设计约束

1. **所有 UI 文案必须走 i18n key**，禁止硬编码中文（`src/i18n/locales/zh-CN.json`）
2. **壁纸库元数据**：存储在 `{app_data_dir}/library.json`，文件存储在 `{app_data_dir}/wallpapers/`
3. **API Key 本地加密存储**，绝不上传
4. **V1 仅支持火山引擎**（DeepSeek/智谱/可灵 灰色待扩展）
5. **关闭窗口 = 隐藏到托盘**，不是退出（`useWindow.ts` 拦截 `onCloseRequested`）
6. **前端路由 Hash 模式**（Tauri 无服务端）
7. **Rust 时间 crate 版本锁定**：`time >=0.3.35, <0.3.37`（避免编译冲突）
