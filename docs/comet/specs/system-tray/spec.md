# system-tray

阶段三：托盘图标 + 关闭拦截 + 唤回主窗口。解决「关窗口直接退出」和「壁纸在后台时主窗无处可寻」两个 polish 项。

## 范围

- `src-tauri/src/lib.rs` 启动时 `build_tray`，注册 `on_window_event` 拦截 `CloseRequested`
- `src-tauri/Cargo.toml` 启用 `tray-icon` / `image-png` features
- `src/components/WinBar.vue` 不需要改

## 条款

### 托盘图标

- MUST：`TrayIconBuilder` 用 `app.default_window_icon().cloned()` 作图标
- MUST：菜单 4 项：`显示灵镜 / 暂停壁纸 / 恢复壁纸 / 退出`
- MUST：左键单击切换主窗口显隐（`window.show() + unminimize() + set_focus()` 或 `window.hide()`）
- MUST：右键菜单默认显示

### 关闭拦截

- MUST：`on_window_event` 截到 `WindowEvent::CloseRequested { api, .. }` 且 `window.label() == "main"` → `window.hide()` + `api.prevent_close()`
- MUST：托盘菜单「退出」走 `app.exit(0)`

### 暂停 / 恢复

- MUST：托盘「暂停壁纸」读 `EngineHandle.state` → `playing = false` → `wallpaper::push_command("pause", ...)`
- MUST：托盘「恢复壁纸」同样设 `playing = true` 并 `push_command("play", ...)`

## 验收

| ID | 条款 | 验证方式 |
|----|------|----------|
| TRAY-1 | 启动后托盘图标出现 | 手动 |
| TRAY-2 | 左键单击切换主窗口显隐 | 手动 |
| TRAY-3 | 主窗口 ✕ 不退出进程 | 手动 |
| TRAY-4 | 托盘「退出」真正退出 | 手动 |
| TRAY-5 | 托盘「暂停壁纸」桌面静态化 | 手动 |
