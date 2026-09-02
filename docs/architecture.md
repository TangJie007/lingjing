# 架构总览

**灵镜 / LingScape**（`com.lingscape.app`）是 Windows-only 桌面应用：  
**Tauri 2 + Vue 3 + TypeScript** 主控面板 + **WebView2** 桌面层壁纸/格子。

## 1. 三窗口模型

配置见 `src-tauri/tauri.conf.json`。

```
┌─────────────────────────────┐
│  main（主窗口）              │  Vue App — 发现/本地/设置/播放条
│  label: main                │  关闭 = 隐藏到托盘（不退出）
└─────────────────────────────┘
              │ invoke / events
              ▼
┌─────────────────────────────┐
│  wallpaper（壁纸层）         │  public/wallpaper.html
│  SetParent → WorkerW        │  在图标层之下播 <video>/<img>
└─────────────────────────────┘
┌─────────────────────────────┐
│  desktop-fence（整理层）     │  fences.html → src/fences/*
│  SetParent → DefView        │  隐藏系统 ListView，自绘格子
└─────────────────────────────┘
```

| Label | URL / 入口 | 职责 |
|-------|------------|------|
| `main` | Vite / `index.html` → `src/main.ts` | 控制面板 UI |
| `wallpaper` | `wallpaper.html` | 动态壁纸渲染（多显示器 tile） |
| `desktop-fence` | `fences.html` → `src/fences/main.ts` | 桌面整理 Fence UI |

托盘、开机自启、`tauri-plugin-single-instance` 挂在主进程（`src-tauri/src/lib.rs`）。

## 2. 逻辑模块

```
                    ┌──────────────┐
                    │   Vue main   │
                    │  views /     │
                    │  composables │
                    └──────┬───────┘
           invoke / listen │
    ┌──────────────────────┼──────────────────────┐
    ▼                      ▼                      ▼
 wallpaper::          settings /              desktop_organize::
 commands + win       library / favorites     + shell_menu
    │                      │                      │
    ▼                      ▼                      ▼
 WorkerW 附着          app_data JSON          DefView 附着
 + wallpaper.html      + 媒体库目录           + fences WebView
                                              + Shell 菜单宿主进程
```

### 2.1 壁纸引擎（`src-tauri/src/wallpaper/`）

- `set_wallpaper` / play / pause → 推状态到 `wallpaper` 窗口
- Win32：`Progman` / `WorkerW` / `SetParent`，多屏 `MonitorTile`
- 前端页 `public/wallpaper.html`：多 pane `<video>`，进度上报 `engine_report_progress`
- 启动恢复：`last_wallpaper.json`

### 2.2 主 UI（`src/`）

| 区域 | 路径 | 说明 |
|------|------|------|
| 壳 | `App.vue`, `IconRail`, `TopBar`, `WinBar`, `PlaybackBar` | 无边框窗、导航、播放 |
| 页 | `views/*` | 本地 / 在线 / 设置 / 关于 / 详情… |
| 状态 | `composables/*` | 引擎、设置、鉴权、HTTP、Toast |
| 样式 | `styles/global.css` | 设计 token 与全局布局 |

路由：`vue-router` hash 模式（`src/router/index.ts`），默认进本地库。

### 2.3 桌面整理（`desktop_organize` + `src/fences/`）

- 启用后：Fence 窗附着 DefView，隐藏系统桌面图标 ListView
- 扫描用户桌面 + 公共桌面，扩展名分桶；系统图标用 `::{CLSID}`
- 打开文件：异步 / 已知文件夹；勿在主路径同步 `ShellExecute`（易与 WorkerW 死锁）
- 右键：内置项走主进程命令；完整 Shell 菜单走独立宿主（`--lingscape-shell-menu-host`）
- 详细对照：[desktop-organize-reference.md](./desktop-organize-reference.md)

### 2.4 Shell 菜单（`src-tauri/src/shell_menu/`）

- 常驻 / one-shot 宿主进程枚举 `IContextMenu`
- 常见 verb（open / properties / runas）优先 `ShellExecute(Ex)`，避免宿主外 `InvokeCommand` 失败
- `runas` 必须允许 UAC UI（不可 `SEE_MASK_FLAG_NO_UI`）

### 2.5 其它 Rust 模块

| 模块 | 职责 |
|------|------|
| `settings.rs` | 设置读写、首启版本记录 |
| `library.rs` | 本地库导入/列表/删除 |
| `favorites.rs` | 收藏 ID |
| `power.rs` | 全屏/电池/会话等自动暂停 + Fence `reassert` |
| `desktop.rs` | 双击隐藏图标钩子、崩溃后图标恢复、无边框 DWM |
| `paths.rs` / `system.rs` / `util.rs` | 路径、低功耗检测、日志 |

## 3. 数据与配置

App data 根：`%APPDATA%/com.lingscape.app/`（以 `identifier` 为准）。

```
app_data_dir/
├── settings.json
├── favorites.json
├── last_wallpaper.json
├── fence_layout.json          # 桌面整理布局（启用时）
└── library/                   # 默认可被 settings 覆盖
    ├── library.json
    └── 媒体文件…
```

临时 / 守护：

- 桌面图标恢复标记：`%TEMP%/lingscape-desktop-icons.guard`
- Shell 菜单宿主：同 exe + 特殊 argv（见 `main.rs` 早退分支）

## 4. 进程与启动

```
lingscape.exe
 ├─ 正常 UI：run() → Tauri Builder
 ├─ --minimized：自启时隐藏主窗
 ├─ --desktop-icons-guard <pid>：守护进程，主进程挂掉后恢复图标
 └─ --lingscape-shell-menu-host …：Shell 菜单宿主（短命或常驻）
```

- **单实例**：`tauri-plugin-single-instance`；二次启动只聚焦已有 `main`
- **关主窗**：`CloseRequested` → `hide`，壁纸与整理继续跑
- **真退出**：托盘「退出」→ `desktop_organize::cleanup` + `wallpaper::cleanup` + `exit`

## 5. 前端 ↔ 后端通信

- **Commands**：`invoke("set_wallpaper" | "open_desktop_item" | …)`
- **Events**：`engine-state`、`navigate-settings`、设置变更通知等
- **HTTP**：在线 API 走 `@tauri-apps/plugin-http`（`apiFetch.ts`），避免 WebView CORS

Capabilities：`src-tauri/capabilities/default.json`（`main` / `wallpaper` / `desktop-fence`）。

## 6. 构建与开发

```bash
pnpm install
pnpm tauri dev          # Vite :1420 + Rust debug
pnpm tauri build        # 安装包
```

技术约束：**仅 Windows**（WorkerW / DefView / OLE / Shell COM）。
