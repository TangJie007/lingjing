# Performance & Power Management (F9)

完整实现 PRD §6 F9 + 阶段三 polish。详细 Rust 端实现见 `power-watch/spec.md`。

## Auto-pause strategies

- 全屏 / 游戏自动暂停：检测前台窗口是否覆盖其所在显示器；pause。
- 电池模式：ACLineStatus == 0 → pause。
- RDP 远程桌面：SM_REMOTESESSION != 0 → pause。
- 任意 paused 状态恢复（离开全屏 / 插电 / 退出 RDP）→ play。

## Performance tier

- 阶段三未做 per-type tier；预留扩展点（`SettingsView` 已有 `pauseOnFullscreen/Battery/Rdp` 三个独立开关）。

## Degradation

- `prefers-reduced-motion`：阶段三未做；UI 仅在 shell-layout 已有体系下生效。
- Low-end：阶段三未做。
- Missing/moved wallpaper file：仍由 `set_wallpaper` / `importMedia` 给出 toast 错误。

## Settings linkage (F8)

- 4 个 set-row 已在 `SettingsView.vue` 接线，见 `settings-system-integration/spec.md` 与 `settings-page/spec.md`。

## Acceptance mapping

- A5（fullscreen / battery / 远程桌面自动暂停 + 恢复）由 power-watch / settings-system-integration 覆盖。
