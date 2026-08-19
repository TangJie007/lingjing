# 灵镜 LINGJING — 动态壁纸

基于 **Tauri 2 + Vue 3 + TypeScript** 的 Windows 动态壁纸桌面应用。壁纸渲染在 WorkerW 桌面层，主窗口为控制面板。

## 功能概览

| 模块 | 能力 |
|------|------|
| **壁纸引擎** | 设壁纸、播放/暂停、音量、多显示器独立 tile、启动恢复上次壁纸 |
| **发现页** | 离线示例 catalog、分类/搜索/排序、详情抽屉 |
| **本地库** | 拖放/文件选择导入、复制或引用、缺失文件检测、删除 |
| **收藏** | `favorites.json` 持久化，重启仍在 |
| **播放条** | 上/下一首、循环模式（列表/单曲/随机，持久化）、音量 |
| **设置** | 开机启动、全屏/电池/RDP 自动暂停、默认音量、壁纸路径迁移 |
| **系统托盘** | 显示/暂停/播放/退出；关闭窗口 → 隐藏到托盘 |

## 技术栈

- **Tauri 2** — Rust 后端 + WebView2 壁纸 worker
- **Vue 3** `<script setup>` + Vite + TypeScript

## 开发

```bash
pnpm install
pnpm tauri dev
```

静默启动（模拟开机自启）：

```bash
pnpm tauri dev -- --minimized
```

## 构建

```bash
pnpm build          # 前端
pnpm tauri build    # Windows 安装包
```

## 数据目录

默认位于 `%APPDATA%/com.lingjing.app/`（具体路径以 Tauri `appIdentifier` 为准）：

```
app_data_dir/
├── settings.json          # 应用设置（含 loopMode、libraryDirOverride 等）
├── favorites.json         # 收藏 ID 列表
├── last_wallpaper.json      # 上次壁纸（用于启动恢复）
└── library/               # 默认本地库（无路径 override 时）
    ├── library.json
    └── *.mp4 / *.png …
```

若在设置中**更改壁纸路径**，`library.json` 与媒体文件会迁移到自定义目录；`settings` / `favorites` / `last_wallpaper` 仍保留在 `app_data_dir`。

## 项目结构

```
├── src/                 # Vue 主窗口 UI
├── src-tauri/           # Rust：引擎、设置、托盘、电源监听
├── public/
│   ├── wallpaper.html   # 桌面壁纸渲染页（多屏 tile）
│   └── samples/         # 示例媒体
└── docs/comet/          # 规格与优化路线图
```

## 相关文档

- [优化路线图](docs/comet/optimization-roadmap.md)
- Phase 2/3 Spec：`docs/comet/specs/`

## 平台

当前仅支持 **Windows**（WorkerW 桌面附着）。
