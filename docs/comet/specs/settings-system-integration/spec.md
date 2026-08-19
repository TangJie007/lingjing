# settings-system-integration (F8)

阶段三：把设置中心 4 个 toggle 从纯 UI 占位升级为真实系统接入。`src/components/SettingsView.vue` 状态来自 `useSettings` composable，写入 `app_data_dir/settings.json`。

## 范围

- `src-tauri/src/settings.rs` 新文件
- `src-tauri/src/lib.rs` 注册 commands `load_settings` / `save_settings`；启动时按 settings 同步 autostart
- `src/components/SettingsView.vue` 5 行 4 toggle + 默认音量滑块
- `src/composables/useSettings.ts` 持久化封装

## 条款

### 持久化

- MUST：`settings.json` 落在 `app_data_dir` 下，结构含 `autostart / hideIconsOnDoubleClick / pauseOnFullscreen / pauseOnBattery / pauseOnRdp / soundOn / defaultVolume / importCopyToData / libraryDirOverride`
- MUST：默认值 `autostart=true, hideIconsOnDoubleClick=false, pauseOnFullscreen=true, pauseOnBattery=true, pauseOnRdp=true, soundOn=true, defaultVolume=0.8, importCopyToData=true, libraryDirOverride=null`

### 开机自启

- MUST：toggle 改动后调 `tauri-plugin-autostart` 的 `enable()` / `disable()`
- MUST：启动时根据 `settings.autostart` 同步一次系统 autostart 状态
- MUST：仅 Windows（macOS / Linux 不实现真实写注册表逻辑，仅保留 toggle 占位）

### 全屏 / 电池 / RDP

- MUST：3 个 toggle 写到 settings；运行期 `power::start_watcher` 每 500ms 读 settings 控制行为
- MUST：监听器见 `docs/comet/specs/power-watch/spec.md`

### 双击隐藏图标

- MUST：toggle 仅持久化到 `settings.json`，UI 切到 on 时弹 toast「依赖后续系统接入」

### 音效

- MUST：toggle 切到 off → `useAudio.soundOn.value = false`；切到 on → `true` 并播放一次示例音

### 默认音量

- MUST：滑块变更时调 `engine_set_volume(volume, muted)`
- MUST：滑块 160×6 圆角 6px；thumb 18×18 圆 50% #fff box-shadow md，left: calc(${vol}% - 9px)

## 验收

| ID | 条款 | 验证方式 |
|----|------|----------|
| SETSYS-1 | settings.json 字段齐 | 静态读 |
| SETSYS-2 | save_settings 命令存在 | 静态读 |
| SETSYS-3 | autostart 真实写注册表 | 手动 |
| SETSYS-4 | 全屏/电池/RDP 监听器存在 | 静态读 |
| SETSYS-5 | 默认音量滑块同步引擎 | 手动 |
| SETSYS-6 | 音效 toggle 即时影响 playClick | 静态读 + 手动 |
