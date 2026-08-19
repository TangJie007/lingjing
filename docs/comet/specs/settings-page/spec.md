# settings-page

设置中心页面：基本设置 4 个 toggle + 自动暂停 2 个 toggle + 壁纸路径 + 默认播放音量滑块。复用 window / sidebar / topbar 框架，仅 main 区内容不同。

## 范围

- `src/components/SettingsView.vue`
- 由 `App.vue` 根据 active nav 切换显示 SettingsView / WallpaperGrid+FavoritesView

## 条款

### 基本设置

- MUST：`<h3>基本设置</h3>` 分组标题
- MUST：4 个 set-row：
  1. 开机启动动态壁纸（默认 on，d：系统启动时自动加载上一次壁纸）— 真实接入 autostart（见 settings-system-integration）
  2. 鼠标双击隐藏桌面图标（默认 off，d：双击桌面空白处隐藏/显示图标）— 仅持久化
  3. 其他程序全屏时变为静态（默认 on）— 接 power-watch
  4. 界面点击音效（默认 on）— 切 useAudio.soundOn
- MUST：每个 set-row 含 `.lead`（`.t` 标题 + `.d` 描述） + `.toggle`

### 自动暂停

- MUST：`<h3>自动暂停</h3>` 分组
- MUST：2 个 set-row：切到电池时暂停 / 远程桌面时暂停（默认 on）

### 路径控件

- MUST：`<h3>壁纸路径</h3>` 分组
- MUST：`<div class="path-ctrl">` 内含 `<input value="..." readonly>` flex 1 + `<div class="btn">更改路径</div>`
- MUST：路径下挂一个「导入时复制到应用数据目录」 toggle，控制 `importCopyToData`
- MUST：「更改路径」触发 `MigrationModal`（见 library-paths）

### 控件设置

- MUST：`<h3>控件设置</h3>` 分组
- MUST：1 个 set-row：默认播放音量 + 滑块
- MUST：滑块 160×6 圆角 6px
- MUST：底部注脚小字 "设置项已接入真实系统设置（部分依赖系统策略）"

### Toggle 组件

- MUST：`.toggle` 44×26 圆角 pill 背景 `var(--border-strong)`
- MUST：`.toggle::after` 圆形 20×20
- MUST：`.toggle.on` 背景 `var(--primary)`，`.toggle.on::after` `left: 21px`
- MUST：toggle `role="switch" tabindex="0" aria-checked="..."` + Enter/Space 触发 flip

## 验收

| ID | 条款 | 验证方式 |
|----|------|----------|
| SET-1 | `src/components/SettingsView.vue` 存在 | 静态读 |
| SET-2 | App.vue 根据 nav active 切换 | 静态读 |
| SET-3 | 4 个基本设置 toggle | grep |
| SET-4 | 2 个自动暂停 toggle | grep |
| SET-5 | 路径 input + 更改路径 btn | grep |
| SET-6 | 复制/引用 toggle | grep |
| SET-7 | 默认音量滑块 | grep |
| SET-8 | 滑块 thumb 18×18 圆 50% | grep |
| SET-9 | 注脚文案 | grep |
| SET-10 | toggle role=switch tabindex=0 aria-checked | grep |
| SET-11 | Enter/Space flip | 静态读 |
