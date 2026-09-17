# 目录结构

仓库根目录与关键路径职责。忽略 `node_modules/`、`src-tauri/target/`、`dist/`。

## 顶层

```
ling-scape/
├── README.md                 # 产品入口、开发/构建、数据目录简述
├── package.json              # 前端依赖与 scripts（dev / build / tauri）
├── vite.config.ts            # Vite + Vue
├── index.html                # 主窗口 HTML
├── fences.html               # 桌面整理窗口 HTML（Vite 多页）
├── public/                   # 静态资源（不经打包变换的路径）
├── src/                      # Vue：主窗口 + fences 前端
├── src-tauri/                # Tauri / Rust
├── docs/                     # 维护文档（本目录）
├── .agents/                  # Agent / Comet 技能与配置
└── .comet/                   # Comet 工作流状态（若使用）
```

## `public/`

| 路径 | 职责 |
|------|------|
| `wallpaper.html` | 壁纸 WebView 页面（多屏 tile、进度上报） |
| `samples/` | 离线示例媒体 |
| `favicon.png` 等 | 图标资源 |

## `src/` — 前端

```
src/
├── main.ts                 # 主窗口入口
├── App.vue                 # 壳：侧栏、路由出口、播放条、全局弹层
├── router/index.ts         # 路由（hash）
├── styles/global.css       # 全局样式与 token
├── views/                  # 页面
│   ├── LocalView.vue
│   ├── OnlineView.vue
│   ├── SettingsView.vue
│   ├── AboutView.vue
│   ├── PetView.vue
│   └── WallpaperDetailView.vue
├── components/             # 主窗口可复用组件
│   ├── IconRail / TopBar / WinBar / PlaybackBar
│   ├── WallpaperCardGrid / MediaThumb / DetailDrawer
│   ├── LoginModal / MigrationModal / Toast …
├── composables/            # 业务状态与 API
│   ├── useEngine.ts        # 壁纸引擎 invoke + engine-state
│   ├── useSettings.ts
│   ├── useAuth.ts / useLingjingApi.ts / apiFetch.ts
│   ├── useToast.ts / useAudio.ts / useVideoPoster.ts
│   └── wallpaperMeta.ts
├── data/catalog.ts         # 离线示例 catalog
├── assets/                 # 图片与 SVG
└── fences/                 # 桌面整理前端（独立入口）
    ├── main.ts             # fences.html 挂载点
    ├── FencesApp.vue
    ├── styles.css
    ├── components/         # FenceCell / FenceGroup / DynamicIsland / Shell 菜单…
    ├── fenceLayout.ts / fenceItems.ts / helpers.ts / types.ts
    ├── iconOpen.ts         # 双击打开、防连点、选中态
    ├── useShellContextMenu.ts / useShellFileDrag.ts / useExternalFileDrop.ts
    └── …
```

主窗口与 Fence **两套 Vue 应用**：共享极少代码；Fence 样式在 `fences/styles.css`，主 UI 在 `global.css`。

## `src-tauri/` — 后端

```
src-tauri/
├── Cargo.toml
├── tauri.conf.json           # 三窗口、identifier、打包
├── capabilities/default.json # 权限
├── icons/
└── src/
    ├── main.rs               # 入口；守护/Shell 宿主早退
    ├── lib.rs                # Builder、托盘、单实例、command 注册
    ├── wallpaper/            # 壁纸引擎
    │   ├── mod.rs / commands.rs / types.rs / win.rs
    ├── desktop_organize/     # 桌面整理
    │   ├── README.md
    │   ├── mod.rs / lifecycle.rs / scan.rs / win.rs
    │   ├── item_commands.rs / menu_commands.rs
    │   ├── drag.rs / drop_target.rs / shell_host.rs
    │   ├── builtin_links.rs / layout.rs / icon_cache.rs
    │   └── …
    ├── shell_menu/           # Shell 右键菜单实现
    │   ├── context.rs / verbs.rs / builtin.rs / host.rs / pin.rs …
    ├── desktop.rs            # 双击藏图标、DWM、图标恢复守卫
    ├── settings.rs / library.rs / favorites.rs
    ├── power.rs / paths.rs / system.rs / util.rs
    └── …
```

### `desktop_organize` 文件速查

| 文件 | 职责 |
|------|------|
| `lifecycle.rs` | 启用/禁用/刷新/监视桌面目录 |
| `scan.rs` | 扫描与分类 |
| `win.rs` | 附着、图标抽取、打开/激活 Explorer |
| `item_commands.rs` | 打开/删/改名/移入文件夹 |
| `menu_commands.rs` | 右键菜单命令分发 |
| `shell_host.rs` | 拉起菜单宿主进程 |
| `drag.rs` / `drop_target.rs` | OLE 拖出 / 拖入（拖出方案见 [fence-ole-drag-out.md](./fence-ole-drag-out.md)） |
| `builtin_links.rs` | 此电脑/回收站/网络 CLSID |

## `docs/`

见 [README.md](./README.md)。`docs/comet/` 为历史规格与归档，**实现以源码为准**；冲突时更新规格或在 maintenance 中注明偏差。

## 配置与脚本对照

| 需求 | 位置 |
|------|------|
| 改窗口尺寸/标题 | `src-tauri/tauri.conf.json` |
| 注册 Tauri command | `src-tauri/src/lib.rs` `invoke_handler` |
| 加主界面路由 | `src/router/index.ts` + `views/` |
| 改壁纸播放页 | `public/wallpaper.html` |
| 改格子 UI | `src/fences/` |
| 改桌面附着/打开策略 | `desktop_organize/win.rs`、`item_commands.rs` |
| 改右键 verb 行为 | `shell_menu/verbs.rs`、`context.rs` |
| HTTP / 在线 API | `composables/apiFetch.ts`、`useLingjingApi.ts` |
| Capability 权限 | `src-tauri/capabilities/default.json` |
