# power-watch (F9)

阶段三：性能 / 电源自动暂停 + 远程桌面检测。后台 tokio 任务，定期采样并 `emit("engine-pause-recommend", { action, reason })`，由前端 `App.vue` 决定是否真正 pause/play。

## 范围

- `src-tauri/src/power.rs` 新文件
- `src-tauri/src/lib.rs` setup 阶段调 `power::start_watcher`
- `src/App.vue` 监听 `engine-pause-recommend`

## 条款

### 全屏检测

- MUST：每 500ms 用 `GetForegroundWindow` + `GetWindowRect` + `GetMonitorInfoW` 判定前台窗口是否覆盖其所在显示器
- MUST：仅在 `settings.pauseOnFullscreen == true` 时启用
- MUST：状态变化才发事件（避免重复 toast）

### 电池模式

- MUST：用 `GetSystemPowerStatus` 读 `ACLineStatus`，0 表示电池
- MUST：切到电池发 `pause { reason: "battery" }`；插电发 `play { reason: "power" }`

### 远程桌面

- MUST：用 `GetSystemMetrics(SM_REMOTESESSION)`；非 0 视为远程会话
- MUST：进入 RDP 发 `pause { reason: "rdp" }`；离开发 `play { reason: "rdp" }`

### 前端响应

- MUST：`App.vue` 收到 `pause` 时调 `engine_pause()` 并 toast「已自动暂停：{reason}」
- MUST：收到 `play` 时调 `engine_play()` 并 toast「已自动恢复播放」
- SHOULD：用户在 1.2s 内主动点过 play/pause，则忽略 pause recommend（避免反复）

## 验收

| ID | 条款 | 验证方式 |
|----|------|----------|
| PWR-1 | start_watcher 启动 | 静态读 + 手动 |
| PWR-2 | 全屏覆盖触发 pause | 手动 |
| PWR-3 | 离开全屏触发 play | 手动 |
| PWR-4 | 电池 ↔ 电源 切换触发对应事件 | 静态读（笔记本） |
| PWR-5 | RDP 进入/离开触发对应事件 | 静态读 |
| PWR-6 | 关闭 toggle 后立即停发 | 手动 |
