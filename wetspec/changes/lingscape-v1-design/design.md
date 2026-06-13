# 技术设计：灵境（LingScape）V1.0

**关联 Spec**：`specs/` 下全部 7 模块 21 功能
**PRD 版本**：AI动态桌面_V1_PRD.md v1.4
**日期**：2026-06-12（壁纸引擎修订：2026-06-13）
**技术栈**：Tauri (Rust) + Vue 3 + Vite
**实现参考**：[`docs/动态桌面实现原理.md`](../../../docs/动态桌面实现原理.md)（Phase 2 壁纸引擎详细说明）

## 1. 目标与范围

### 做什么

- Windows 桌面应用：AI 动态壁纸创作 + 播放 + 管理
- 两步 AI 管线：VL-LLM 意图分析 → Seedream 壁纸生成
- 纯 BYOK：用户自带火山引擎 API Key，零服务器
- 国际化框架：本期 zh-CN，预留多语言
- Steam 分发：¥9.9 买断制

### 不做什么

- 不做云端代理/后端服务器
- 不做 macOS / Linux（V3.0+）
- 不做 AI 实时壁纸演变（V2.0）
- 不做创意工坊 / UGC（V2.0）

## 2. 架构决策

| 决策 | 选择 | 理由 |
|------|------|------|
| 桌面框架 | **Tauri 2.x** | 安装包 < 10MB（vs Electron 150MB+），Rust 性能极致，原生 Win32 API 可直接调 |
| 前端框架 | **Vue 3 (Composition API)** | 生态成熟、学习曲线平缓、响应式系统适合复杂 UI 状态 |
| 构建工具 | **Vite 5** | 开发热更新 < 1s，生产 Tree-shaking 优秀 |
| 状态管理 | **Pinia** | Vue 3 官方推荐，TypeScript 友好 |
| UI 组件库 | **自建 + Tailwind CSS** | 桌面软件 UI 不走 Web 组件库（太重），轻量 Tailwind + 自建组件 |
| 路由 | **Vue Router 4** | Hash 模式（Tauri 无服务端） |
| 国际化 | **vue-i18n 10** | Vue 生态标准方案，支持热切换、懒加载 |
| 测试框架 | **Vitest** | 与 Vite 共享配置，速度快，Vue 生态首选 |
| HTTP 客户端 | **Tauri HTTP Plugin** (Rust 侧) | 可复用 Rust 侧 TLS 栈，API Key 不暴露到前端 JS |
| 壁纸渲染 | **Win32 桌面层嵌入** (Rust) | 探测 Progman / WorkerW / shell_host，将播放窗口嵌入图标层之下 |
| 视频解码 | **MPV 外部进程** (`--wid`) | 参考 Lively / weebp；硬件解码由 MPV 负责，避免在 Rust 内集成 ffmpeg 绑定 |
| 加密存储 | **Rust `ring` crate + AEAD** | API Key 本地 AES-256-GCM 加密 |
| 全屏检测 | **Win32 轮询** (Rust) | `GetForegroundWindow` + `GetWindowRect` 比对显示器尺寸（3s 间隔） |
| 开机自启 | **Windows 注册表 Run key** | 标准方案，Tauri 提供 plugin |

## 2.1 设计准则

以下准则在全部代码编写中**强制遵守**，Code Review 时逐条检查：

| # | 准则 | 说明 |
|---|------|------|
| 1 | **单一职责** | 每个模块/文件/函数只做一件事。Vue 组件：一个组件只负责一个 UI 区域或一个交互单元；Rust 模块：一个 `.rs` 文件只负责一个明确的领域（如 `crypto.rs` 只管加解密，不混入 HTTP 或文件 I/O）；Pinia store：一个 store 只管理一个业务域的状态。 |
| 2 | **不过度设计** | V1 只做 Spec 明确要求的功能。不做"以后可能需要"的抽象层、不做多余的接口定义、不做未在 PRD 中出现的配置项。3 行能解决的问题不写 30 行的设计模式。 |
| 3 | **文件行数上限 500 行** | 任何 `.rs` / `.ts` / `.vue` 文件超过 500 行时必须考虑拆分。Rust — 按职责拆为新模块/文件；Vue — 提取子组件或 composable；TypeScript — 按功能域拆文件。超过 500 行且确认无法合理拆分时，在文件头部注释说明原因。 |
| 4 | **优先第三方库** | 遇到工具方法、功能方法时，先查找是否有成熟第三方库可用。选择标准：① npm/crates.io 上最近 3 个月内有更新；② 周下载量 > 10k（npm）或总下载 > 50k（crates.io）；③ 非个人开发者单维护（2+ 贡献者或知名组织）。**不满足以上标准的库，必须先找我确认再引入。** 找不到合适库时才手写，手写时在函数头部注释说明"已调研 xxx 库但不满足条件，因此手写"。 |
| 5 | **关键代码必须注释** | 以下场景必须有清晰注释：① 每个 Rust 模块顶部 `//!` 说明模块职责和设计意图；② 每个公开函数/方法有 doc comment 说明参数、返回值、副作用；③ 复杂算法或非直觉的逻辑（如 Win32 API 调用、位运算、状态机转换）逐行注释；④ 使用第三方库的非典型用法时注释"为什么这样用"。注释语言：Rust 用英文，Vue/TS 用中文。**注释不是越多越好——简单的 getter/setter、一眼能看懂的赋值不需要注释。** |

### 2.1.1 拆分阈值速查

| 语言 | 文件上限 | 拆分信号 | 拆分方式 |
|------|---------|---------|---------|
| Rust `.rs` | 500 行 | struct/trait/impl 超过 3 组、函数超过 15 个 | 按领域拆为新模块文件 |
| Vue `.vue` | 500 行（含 template+script+style） | template 超过 150 行 或 script 超过 250 行 | 提取子组件或 composable |
| TypeScript `.ts` | 500 行 | 导出符号超过 20 个 | 按功能域拆文件 |

### 2.1.2 第三方库白名单（预审通过，可直接使用）

以下库经预先审查满足维护标准，开发时可直接引入：

| 类别 | 库名 | 版本 | 用途 |
|------|------|------|------|
| Vue 生态 | `vue` `vue-router` `pinia` `vue-i18n` | latest stable | 框架核心 |
| 构建 | `vite` `@vitejs/plugin-vue` `typescript` | latest stable | 构建工具链 |
| CSS | `tailwindcss` `postcss` `autoprefixer` | latest stable | 样式 |
| 测试 | `vitest` `@vue/test-utils` `jsdom` | latest stable | 单元测试 |
| 工具 | `@vueuse/core` | latest stable | Vue composables 工具集（防抖、事件监听、localStorage 等） |
| Rust HTTP | `reqwest` | latest stable | Rust 侧 HTTP 客户端 |
| Rust 加密 | `ring` | latest stable | AEAD 加密 |
| Rust 序列化 | `serde` `serde_json` | latest stable | JSON 序列化 |
| 视频播放 | **MPV**（外部可执行文件） | 系统安装或 `bin/mpv/mpv.exe` | 视频壁纸解码与循环播放（`--hwdec=auto-safe`） |

> 原设计中的 `ffmpeg-next` 未采用——Rust 侧 ffmpeg 绑定维护成本高，且 Lively 等同类产品均使用 MPV `--wid` 方案。不在白名单中的库，引入前须按准则 4 验证并找我确认。

## 3. 项目结构

```
lingscape/
├── src-tauri/                  # Rust 后端 (Tauri)
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── src/
│   │   ├── main.rs             # 入口
│   │   ├── lib.rs              # 核心逻辑
│   │   ├── wallpaper/          # 壁纸渲染引擎 (WP-001~003)
│   │   │   ├── mod.rs
│   │   │   ├── static.rs       # 静态图片渲染
│   │   │   ├── video.rs        # 视频壁纸播放
│   │   │   └── fullscreen.rs   # 全屏检测
│   │   ├── api/                # HTTP 请求 (AI-001~002)
│   │   │   ├── mod.rs
│   │   │   ├── doubao.rs       # 豆包 VL-LLM 调用
│   │   │   └── seedream.rs     # Seedream 图片生成
│   │   ├── crypto/             # API Key 加密存储 (API-001)
│   │   │   └── mod.rs
│   │   ├── tray/               # 系统托盘 (ST-001)
│   │   │   └── mod.rs
│   │   ├── autostart/          # 开机自启 (SET-001)
│   │   │   └── mod.rs
│   │   └── commands.rs         # Tauri IPC 命令注册
│   └── icons/                  # 应用图标
│
├── src/                        # Vue 3 前端
│   ├── main.ts                 # 入口
│   ├── App.vue                 # 根组件
│   ├── router/
│   │   └── index.ts            # 路由配置
│   ├── stores/                 # Pinia 状态管理
│   │   ├── app.ts              # 全局状态
│   │   ├── ai-pipeline.ts      # AI 管线状态 (AI-001~003)
│   │   ├── wallpaper.ts        # 壁纸引擎状态 (WP-001~003)
│   │   ├── library.ts          # 壁纸库状态 (WL-001~002)
│   │   ├── api-keys.ts         # API Key 状态 (API-001~003)
│   │   └── settings.ts         # 设置状态 (SET-001~004)
│   ├── i18n/                   # 国际化 (SET-005)
│   │   ├── index.ts            # 初始化 + setLocale
│   │   ├── locales/
│   │   │   ├── zh-CN.json      # 中文简体（完整）
│   │   │   └── en.json         # 英文（骨架）
│   │   └── formatters.ts       # Intl 格式化封装
│   ├── config/                 # 常量配置
│   │   └── constants.ts
│   ├── views/                  # 页面
│   │   ├── OnboardingPage.vue  # 首次向导 (OB-001~004)
│   │   ├── AiCreatePage.vue    # AI 创作页 (AI-001~003)
│   │   ├── LibraryPage.vue     # 壁纸库 (WL-001~002)
│   │   ├── ApiKeysPage.vue     # API 管理 (API-001~003)
│   │   └── SettingsPage.vue    # 设置页 (SET-001~004)
│   ├── components/             # 通用组件
│   │   ├── layout/
│   │   │   ├── AppSidebar.vue
│   │   │   ├── AppTitlebar.vue
│   │   │   └── AppStatusbar.vue
│   │   ├── ai/
│   │   │   ├── PromptInput.vue
│   │   │   ├── ReferenceUpload.vue
│   │   │   ├── StyleSelector.vue
│   │   │   ├── PlanCard.vue
│   │   │   ├── GenerationProgress.vue
│   │   │   └── ResultGrid.vue
│   │   ├── wallpaper/
│   │   │   ├── WallpaperPreview.vue
│   │   │   └── ScalingSelector.vue
│   │   ├── library/
│   │   │   ├── WallpaperGrid.vue
│   │   │   └── HistoryList.vue
│   │   ├── api-keys/
│   │   │   ├── PlatformCard.vue
│   │   │   ├── KeyInput.vue
│   │   │   └── StatusIndicator.vue
│   │   ├── onboarding/
│   │   │   ├── WelcomeStep.vue
│   │   │   ├── ApiGuideStep.vue
│   │   │   └── QuickStartStep.vue
│   │   └── common/
│   │       ├── Toast.vue
│   │       └── SettingsToggle.vue
│   └── assets/
│       └── styles/
│           └── main.css        # Tailwind + 全局样式
│
├── __tests__/                  # 前端单元测试 + AC 验收
│   ├── stores/                 # Pinia store 测试
│   ├── components/             # 组件测试
│   └── helpers/                # 工具函数测试
│
├── vite.config.ts
├── tsconfig.json
├── tailwind.config.js
├── vitest.config.ts
├── package.json
└── index.html
```

## 3.1 模块 slug 映射

| PRD 中文模块 | 代码目录 (slug) |
|-------------|----------------|
| AI生成管线 | `src-tauri/src/api/` + `src/views/AiCreatePage.vue` |
| API Key管理 | `src-tauri/src/crypto/` + `src/views/ApiKeysPage.vue` |
| 壁纸播放引擎 | `src-tauri/src/desktop_core.rs` + `video_player.rs` + `wallpaper_engine.rs` |
| 壁纸库管理 | `src/views/LibraryPage.vue` + `src/stores/wallpaper.ts` |
| 系统托盘 | `src-tauri/src/tray.rs` |
| 首次启动向导 | `src/views/OnboardingPage.vue` |
| 基础设置 | `src/views/SettingsPage.vue` + `src/stores/settings.ts` |

### 3.2 壁纸引擎实际结构（Phase 2，偏离原 §3 规划）

原规划 `src-tauri/src/wallpaper/{static,video,fullscreen}.rs` 未按子目录拆分，当前以平铺模块实现：

```
src-tauri/src/
├── desktop_core.rs      # 桌面层探测、WorkerW、AttachMode、Z-order、第三方冲突检测
├── video_player.rs      # MPV 启停、多屏 --wid、看门狗、进程清理
├── wallpaper_engine.rs  # 壁纸入口、Windows SPI、库 CRUD、全屏检测
├── tray.rs
├── autostart.rs
└── lib.rs               # 命令注册、退出时停止 MPV
```

前端：`src/stores/wallpaper.ts` 统一管理库列表与当前壁纸；`DesktopPlayer.vue`（WebView 播放器）保留备用，**当前视频壁纸不走此路径**。

## 4. 数据流

### 4.1 AI 生成管线

```
用户输入描述 + 参考图 (Vue)
    │
    ▼
Pinia store ai-pipeline.ts  ──→ invoke('ai_analyze', {prompt, image?})
    │                                    │
    │                              Tauri Command (Rust)
    │                                    │
    │                              api/doubao.rs
    │                              → POST 火山引擎 doubao API
    │                              ← 结构化构建方案 JSON
    │                                    │
    ▼                                    ▼
PlanCard.vue 展示方案            返回给前端
    │
    ▼ (用户确认/编辑后)
invoke('ai_generate', {prompt, count, resolution})
    │
    ▼
api/seedream.rs
→ POST 火山引擎 Seedream API
← 图片 URL / base64 数组
    │
    ▼
ResultGrid.vue 展示变体 → 用户选一张 → invoke('set_wallpaper', { id })
    │
    ▼
wallpaper_engine.rs → 静态: SPI_SETDESKWALLPAPER；视频: MPV --wid 嵌入桌面层
```

### 4.2 API Key 存储流

```
用户输入 Key (Vue KeyInput.vue)
    │
    ▼
invoke('crypto_encrypt', {plaintext})  ← 绝不在前端存储明文
    │
    ▼
crypto/mod.rs → ring AEAD 加密 → 写入本地文件
    │
    ▼ (后续使用时)
invoke('crypto_decrypt') → 解密 → 拼入 API 请求头
```

### 4.3 壁纸播放流

```
壁纸应用 (LibraryPage → wallpaperStore.setWallpaper)
    │
    ▼
invoke('set_wallpaper', { id })
    │
    ▼
wallpaper_engine::set_wallpaper()
    ├─ detect_desktop_layer_conflicts()     // 仅检测，不修改第三方窗口
    │     └─ 有冲突 → 返回 conflicts[]，前端 warning toast
    │
    ├── 静态图片:
    │     stop_video_wallpaper()
    │     hide_desktop_player_window()
    │     SystemParametersInfoW(SPI_SETDESKWALLPAPER)
    │
    └── 视频:
          hide_desktop_player_window()
          run_on_main_thread → video_player::start_video_wallpaper()
            ├─ setup_desktop_layer()        // 0x052C 创建 WorkerW
            ├─ resolve_attach_mode()        // ClassicWorkerW / ShellHostLayered / RaisedProgman
            ├─ enumerate_monitors()         // 每显示器启动一个 MPV
            │     mpv --wid={HWND} --geometry={w}x{h}+{x}+{y} --loop-file=inf ...
            ├─ position_player_in_wallpaper_worker()  // 铺满对应屏
            ├─ ensure_desktop_zorder()      // 图标在壁纸之上
            └─ start_desktop_watchdog()     // 3s 轮询：WorkerW 丢失 / MPV 消失时 reattach
    │
    ▼
返回 SetWallpaperResult { conflicts }
```

**第三方桌面工具（如腾讯桌面整理）**：检测到 `TXMiniSkin` 全屏覆盖或相关进程时提示用户关闭，壁纸仍尝试启动。不自动隐藏第三方窗口（与飞火动态壁纸等产品策略一致）。

**全屏暂停**（WP-003）：`is_fullscreen_app_running()` 轮询前台窗口尺寸；状态位供 UI / 托盘使用，MPV IPC 暂停待后续接入。

## 4.1 测试策略

| 用途 | 框架 | 配置来源 |
|------|------|----------|
| Rust 单元测试 | `cargo test` | `src-tauri/Cargo.toml` |
| 前端实现 + AC 验收 | Vitest | `specs/.wetspec.yaml` → `unit_test` |

build：`npm run test:unit`（Vitest）；verify：`wetspec verify <spec.yaml> --root .`（写回 YAML）。

AC 测试嵌套约定（Vitest）：
```
describe('AI-001', () => {
  describe('AC-001: 纯文字描述分析', () => {
    it('在3秒内返回优化后的英文prompt且风格标签准确', () => { ... })
  })
  describe('AC-002: 文字+参考图分析', () => {
    it('正确分析参考图的构图/色调/风格', () => { ... })
  })
})
```

## 5. 配置与常量

| 名称 | 值 | 来源 |
|------|-----|------|
| `DOUBAO_VISION_MODEL` | `doubao-2.0-vision` | AI-001 §模型 |
| `DOUBAO_LITE_MODEL` | `doubao-2.0-lite-32k` | AI-001 §模型 |
| `SEEDREAM_MODEL` | `Seedream 4.0` | AI-002 §模型 |
| `SEEDREAM_PREMIUM_MODEL` | `Seedream 5.0 lite` | AI-002 §模型 |
| `JIMENG_VIDEO_MODEL` | `即梦视频 3.0 Pro` | AI-002 §模型 |
| `AI_STEP1_TIMEOUT_MS` | `3000` | AI-001 AC-001 |
| `AI_STEP2_TIMEOUT_MS` | `10000` | AI-002 AC-001 |
| `DEFAULT_GENERATION_COUNT` | `3` | SET-002 |
| `DEFAULT_RESOLUTION` | `1080P` | SET-002 |
| `MAX_GENERATION_COUNT` | `5` | AI-002 |
| `VIDEO_FPS_DEFAULT` | `30` | SET-003 |
| `VIDEO_CPU_MAX_PCT` | `3` | WP-002 AC-001 |
| `STATIC_CPU_MAX_PCT` | `0.5` | WP-001 AC-001 |
| `WALLPAPER_CACHE_LIMIT_GB` | `5` | SET-004 |
| `API_KEY_MASK_PREFIX` | `sk-****` | API-001 AC-002 |
| `DEFAULT_LOCALE` | `zh-CN` | SET-005 |
| `FALLBACK_LOCALE` | `zh-CN` | SET-005 AC-005 |
| `I18N_BUNDLE_MAX_KB` | `50` | SET-005 NFR |
| `ONBOARDING_WELCOME_DURATION_MS` | `3000` | OB-001 AC-001 |
| `ONBOARDING_QUICKSTART_DURATION_MS` | `60000` | OB-003 AC-001 |
| `VOLCANO_ENGINE_BASE_URL` | `https://ark.cn-beijing.volces.com/api/v3` | API-001 |
| `VOLCANO_REGISTER_URL` | `https://console.volcengine.com/ark/region:ark+cn-beijing/overview` | OB-002 |

## 6. API / 接口

### 6.1 Tauri IPC 命令（Rust → 前端）

> 壁纸相关命令以 **当前实现** 为准；AI / 加密等模块命令仍为设计规划，待后续迭代对齐。

| 命令 | 方向 | 用途 | 关联 Spec |
|------|------|------|-----------|
| `list_wallpapers` | invoke | 获取壁纸库列表 | WL-001 |
| `import_wallpaper` | invoke | 导入本地壁纸文件 | WL-001 |
| `delete_wallpaper` | invoke | 删除壁纸 | WL-001 |
| `export_wallpaper` | invoke | 导出壁纸 | WL-001 |
| `set_wallpaper` | invoke | 设置壁纸，返回 `{ conflicts }` | WP-001/002 |
| `check_desktop_layer_conflicts` | invoke | 检测桌面层级冲突（第三方工具） | WP-002 |
| `get_current_wallpaper` | invoke | 获取当前壁纸状态 | WP-001/002 |
| `get_library_size` | invoke | 壁纸库占用空间 | SET-004 |
| `clear_library` | invoke | 清除壁纸库 | SET-004 |
| `is_fullscreen_app_running` | invoke | 检测是否有全屏应用 | WP-003 |
| `attach_desktop_player` | invoke | 重新附着 WebView 播放器（备用） | — |
| `wallpaper-changed` | event | 通知壁纸变更（含 `engine: "mpv"`） | WP-002 |
| `ai_analyze` | invoke | 第一步：VL-LLM 意图分析 | AI-001 |
| `ai_generate` | invoke | 第二步：Seedream 壁纸生成 | AI-002 |
| `crypto_encrypt` | invoke | 加密存储 API Key | API-001 |
| `crypto_decrypt` | invoke | 解密读取 API Key | API-001 |
| `crypto_test_connection` | invoke | 测试 API Key 有效性 | API-002 |
| `tray_menu_action` | event | 托盘菜单点击 | ST-001 |
| `autostart_set` | invoke | 设置开机自启 | SET-001 |
| `autostart_get` | invoke | 读取开机自启状态 | SET-001 |
| `open_url` | invoke | 系统默认浏览器打开链接 | OB-002 |

### 6.2 外部 API

| 服务 | 端点 | 用途 | 关联 Spec |
|------|------|------|-----------|
| 火山引擎 | `/chat/completions` | doubao VL-LLM 对话 | AI-001 |
| 火山引擎 | `/images/generations` | Seedream 图片生成 | AI-002 |
| 火山引擎 | `/videos/generations` | 即梦视频生成 | AI-002 |
| 火山引擎 | `/models` (最小请求) | API Key 有效性检测 | API-002 |

## 7. 验收映射

### AI 生成管线

| AC ID | 设计要点 | 测试嵌套 |
|-------|----------|----------|
| AI-001 AC-001 | doubao-lite 纯文字分析 < 3s | `describe('AI-001')` → `describe('AC-001: 纯文字描述分析')` |
| AI-001 AC-002 | doubao-vision 文字+参考图分析 | `describe('AI-001')` → `describe('AC-002: 文字+参考图分析')` |
| AI-001 AC-003 | API Key 无效 → 提示跳转设置 | `describe('AI-001')` → `describe('AC-003: API Key无效处理')` |
| AI-001 AC-004 | 网络错误 → 重试按钮 | `describe('AI-001')` → `describe('AC-004: 网络错误处理')` |
| AI-002 AC-001 | Seedream 生成 3 张 1080P < 10s | `describe('AI-002')` → `describe('AC-001: Seedream 4.0生成壁纸')` |
| AI-002 AC-002 | 生成中进度动画 | `describe('AI-002')` → `describe('AC-002: 生成中显示进度')` |
| AI-002 AC-003 | 结果网格展示 + 操作 | `describe('AI-002')` → `describe('AC-003: 生成结果展示与操作')` |
| AI-002 AC-004 | 余额不足 → Toast + 充值链接 | `describe('AI-002')` → `describe('AC-004: API余额不足处理')` |
| AI-002 AC-005 | 内容审核拦截 → 提示修改 | `describe('AI-002')` → `describe('AC-005: 内容审核拦截处理')` |
| AI-002 AC-006 | 失败自动重试一次 | `describe('AI-002')` → `describe('AC-006: 生成失败自动重试')` |
| AI-003 AC-001 | 方案卡片展示 prompt/风格/色调 | `describe('AI-003')` → `describe('AC-001: 方案卡片展示')` |
| AI-003 AC-002 | Prompt 手动编辑 | `describe('AI-003')` → `describe('AC-002: Prompt手动编辑')` |
| AI-003 AC-003 | 确认后触发第二步 | `describe('AI-003')` → `describe('AC-003: 确认后触发生成')` |

### API Key 管理

| AC ID | 设计要点 | 测试嵌套 |
|-------|----------|----------|
| API-001 AC-001 | 完整配置流程 < 2 分钟 | `describe('API-001')` → `describe('AC-001: 完整配置流程')` |
| API-001 AC-002 | Key 掩码显示 | `describe('API-001')` → `describe('AC-002: API Key掩码显示')` |
| API-001 AC-003 | 粘贴自动去空格 | `describe('API-001')` → `describe('AC-003: 粘贴自动去空格')` |
| API-001 AC-004 | 本地加密存储 | `describe('API-001')` → `describe('AC-004: 本地加密存储')` |
| API-002 AC-001 | 测试连接成功 → 绿灯 | `describe('API-002')` → `describe('AC-001: 测试连接成功')` |
| API-002 AC-002 | 测试连接失败 → 红灯 | `describe('API-002')` → `describe('AC-002: 测试连接失败')` |
| API-002 AC-003 | 未配置 → 灰灯 | `describe('API-002')` → `describe('AC-003: 未配置状态')` |

### 壁纸播放引擎

| AC ID | 设计要点 | 测试嵌套 |
|-------|----------|----------|
| WP-001 AC-001 | JPG/PNG/WebP 经 `SPI_SETDESKWALLPAPER`，CPU < 0.5% | `describe('WP-001')` → `describe('AC-001: 静态图片壁纸播放')` |
| WP-002 AC-001 | MP4 1080P MPV 硬件解码 CPU < 3%；多屏每显示器一实例 | `describe('WP-002')` → `describe('AC-001: 视频壁纸播放')` |
| WP-002 AC-002 | 第三方桌面整理占用层级时 warning toast，不阻断启动 | `describe('WP-002')` → `describe('AC-002: 桌面层级冲突提示')` |
| WP-003 AC-001 | 全屏应用检测；暂停状态供 UI 展示（MPV IPC 待接） | `describe('WP-003')` → `describe('AC-001: 全屏检测自动暂停')` |

### 壁纸库管理

| AC ID | 设计要点 | 测试嵌套 |
|-------|----------|----------|
| WL-001 AC-001 | 网格展示 + 应用/删除/导出 | `describe('WL-001')` → `describe('AC-001: 本地壁纸管理')` |
| WL-002 AC-001 | 历史记录展示 prompt/模型/时间/费用 | `describe('WL-002')` → `describe('AC-001: AI生成历史')` |

### 系统托盘

| AC ID | 设计要点 | 测试嵌套 |
|-------|----------|----------|
| ST-001 AC-001 | 最小化托盘 + 右键菜单 | `describe('ST-001')` → `describe('AC-001: 托盘最小化与右键菜单')` |

### 首次启动向导

| AC ID | 设计要点 | 测试嵌套 |
|-------|----------|----------|
| OB-001 AC-001 | 品牌动画 3 秒可跳过 | `describe('OB-001')` → `describe('AC-001: 欢迎页与品牌动画')` |
| OB-002 AC-001 | 引导内容完整展示 | `describe('OB-002')` → `describe('AC-001: 火山引擎注册引导')` |
| OB-003 AC-001 | 预设描述 → 30-60s 完成生成 | `describe('OB-003')` → `describe('AC-001: 快速上手与一键生成')` |
| OB-004 AC-001 | 跳过配置 → 本地模式 + 引导条 | `describe('OB-004')` → `describe('AC-001: 跳过API配置的本地模式')` |

### 基础设置

| AC ID | 设计要点 | 测试嵌套 |
|-------|----------|----------|
| SET-001 AC-001 | 开机自启开关持久化 | `describe('SET-001')` → `describe('AC-001: 开机自启开关')` |
| SET-001 AC-002 | 语言切换热生效 | `describe('SET-001')` → `describe('AC-002: 语言切换热生效')` |
| SET-001 AC-003 | 下拉仅中文简体可选 | `describe('SET-001')` → `describe('AC-003: 语言选项本期仅中文简体')` |
| SET-002 AC-001 | API 默认值持久化 | `describe('SET-002')` → `describe('AC-001: API默认值与生成配置')` |
| SET-003 AC-001 | 帧率/缩放实时生效 | `describe('SET-003')` → `describe('AC-001: 壁纸播放设置')` |
| SET-004 AC-001 | 存储上限/缓存清除/全屏暂停 | `describe('SET-004')` → `describe('AC-001: 性能与存储设置')` |
| SET-005 AC-001 | 无硬编码中文字符串 | `describe('SET-005')` → `describe('AC-001: 无硬编码中文')` |
| SET-005 AC-002 | zh-CN 完整覆盖 | `describe('SET-005')` → `describe('AC-002: zh-CN 完整覆盖')` |
| SET-005 AC-003 | en.json 骨架存在 | `describe('SET-005')` → `describe('AC-003: en 骨架文件存在')` |
| SET-005 AC-004 | setLocale 热切换 | `describe('SET-005')` → `describe('AC-004: 语言热切换')` |
| SET-005 AC-005 | 缺失 key 回退 zh-CN | `describe('SET-005')` → `describe('AC-005: 缺失 key 回退')` |
| SET-005 AC-006 | Intl 格式化正确 | `describe('SET-005')` → `describe('AC-006: 格式化方法')` |

## 8. 风险与回滚

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| Win11 24H2 Raised Desktop / WorkerW 结构变化 | 壁纸嵌入失败 | `AttachMode` 三路径回退（ClassicWorkerW → RaisedProgman → ShellHostLayered）；`scripts/diag-*.ps1` 诊断 |
| 第三方桌面整理（腾讯等）遮挡壁纸层 | 动态壁纸不可见 | **仅检测 + 提示用户关闭**，不自动干预其窗口；与同类产品策略一致 |
| 用户未安装 MPV | 视频壁纸无法启动 | 启动前检测 `mpv.exe`；提示安装或放置到 `bin/mpv/` |
| 火山引擎 API 变更 | AI 管线中断 | API 调用集中在 Rust 侧两个文件，变更影响面小；保留 `--model` 参数可配置 |
| 视频解码 CPU 占用超标 | 办公场景体验差 | MPV `--hwdec=auto-safe`；后续可接 MPV IPC 降帧率 |
| vue-i18n 热切换导致组件重渲染闪烁 | UI 体验下降 | 使用 `<i18n-t>` 组件 + key 作为过渡标识；必要时降级为整页刷新模式 |
| Steam 审核周期超预期 | 错过 Q4 窗口 | 提前 2 个月提交商店页审核，期间继续迭代功能 |
