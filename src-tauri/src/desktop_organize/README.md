# desktop_organize

桌面整理（Fence 窗口）后端模块，由原单文件 `desktop_organize.rs`（~2600 行）拆分而来。

## 模块结构

| 模块 | 职责 |
|------|------|
| `types.rs` | `DesktopItem`、`ShellMenuEntry` 数据结构 |
| `state.rs` | 全局状态（ACTIVE、WATCH_GEN、FENCE_LABEL） |
| `util.rs` | `run_on_ui` 等通用工具 |
| `scan.rs` | 桌面目录扫描、分类、图标元数据 |
| `win.rs` | Windows 专用：Shell 图标、Fence 窗口附着/隐藏 |
| `lifecycle.rs` | 启用/禁用/刷新/清理、文件监视 |
| `shell_host.rs` | Shell 菜单独立子进程（`--lingscape-shell-menu-host`） |
| `item_commands.rs` | Tauri 命令：打开/删除/重命名/设整理开关等 |
| `menu_commands.rs` | Tauri 命令：Shell 右键菜单枚举与执行 |
| `drag.rs` | Tauri 命令：拖出到外部窗口（CF_HDROP） |
| `mod.rs` | 模块入口与对外 re-export |

## 对外接口

- `lib.rs` 通过 `desktop_organize::item_commands::*` 等子模块注册 Tauri 命令
- `shell_menu.rs` 依赖 `ShellMenuEntry` 与 `shell_host_stage`
- `power.rs` 调用 `reassert`

## 后续可继续拆分

- `win.rs` 仍可拆为 `win_icons.rs` + `win_fence.rs`
- `menu_commands.rs` 中的内置命令 helpers 可抽到 `item_ops.rs`
