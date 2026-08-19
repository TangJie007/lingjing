# settings-page

设置中心页面：基本设置 4 个 toggle + 壁纸路径 + 默认播放音量滑块。复用 window / sidebar / topbar 框架，仅 main 区内容不同。

## 范围

- `src/components/SettingsView.vue` 新增
- 由 `App.vue` 根据 active nav 切换显示 SettingsView / WallpaperGrid+FavoritesView
- 复用 `IconRail` / 窗口外壳，不重写

## 条款

### 基本设置

- MUST：`<h3>基本设置</h3>` 分组标题，font-size 15 font-weight 700 margin-bottom 6px
- MUST：4 个 set-row：
  1. 开机启动动态壁纸（默认 on，d：系统启动时自动加载上一次壁纸）
  2. 鼠标双击隐藏桌面图标（默认 off，d：双击桌面空白处隐藏/显示图标）
  3. 其他程序全屏时变为静态（默认 on，d：节省资源，游戏/观影更流畅）
  4. 界面点击音效（默认 on，d：点击导航与操作时的轻量反馈音；this 切换全局 `soundOn` 状态）
- MUST：每个 set-row 含 `.lead`（`.t` 标题 + `.d` 描述，font-size 12 color `var(--text-3)` margin-top 2px） + `.toggle`

### 路径控件

- MUST：`<h3>壁纸路径</h3>` 分组
- MUST：`<div class="path-ctrl">` 内含 `<input value="..." readonly>` flex 1 + `<div class="btn">更改路径</div>`
- MUST：input 背景 `var(--surface-2)`，focus 时 border `var(--primary)` + box-shadow `0 0 0 3px var(--primary-soft)`
- MUST：btn hover `background: var(--surface-2)`，按下 `transform: scale(.96)`

### 控件设置

- MUST：`<h3>控件设置</h3>` 分组
- MUST：1 个 set-row：默认播放音量 + 滑块
- MUST：滑块 160×6 圆角 6px 背景 `var(--border)`；滑块填充条 `position: absolute; left 0; top 0; height 100%; width: ${vol}%`，背景 `var(--primary)` 圆角 6px
- MUST：滑块 thumb `position: absolute; left: calc(${vol}% - 9px); top: -6px`，18×18 圆角 50% 背景 #fff box-shadow `var(--sh-md)` border 1px `var(--border)`
- MUST：底部注脚小字 "设置项将在 Phase 3 接入真实系统设置"（居中 color `var(--text-3)` font-size 11）

### Toggle 组件

- MUST：`.toggle` 44×26 圆角 pill 背景 `var(--border-strong)`，position relative cursor pointer
- MUST：`.toggle::after` 圆形 20×20 `position: absolute; top: 3px; left: 3px` 背景 #fff box-shadow `var(--sh-sm)` 圆角 50%
- MUST：`.toggle::after` 过渡 `left var(--dur-base) var(--ease)`
- MUST：`.toggle.on` 背景 `var(--primary)`，`.toggle.on::after` `left: 21px`
- MUST：toggle `role="switch" tabindex="0" aria-checked="..."` + Enter/Space 触发 flip
- MUST：flip 时切换 `aria-checked` 同步状态

## 验收

| ID | 条款 | 验证方式 |
|----|------|----------|
| SET-1 | `src/components/SettingsView.vue` 存在 | 静态读 |
| SET-2 | App.vue 根据 nav active 切换 WallpaperGrid / SettingsView / FavoritesView | 静态读 |
| SET-3 | `<h3>基本设置</h3>` 4 个 set-row | grep |
| SET-4 | 4 个 toggle：开机启动/双击隐藏/全屏静态/界面音效 | grep |
| SET-5 | 默认 on: 开机启动/全屏静态/界面音效；默认 off: 双击隐藏 | 静态读 |
| SET-6 | 界面音效 toggle 切换 `soundOn` 全局状态 | 静态读 |
| SET-7 | `<h3>壁纸路径</h3>` + path-ctrl input + 更改路径 btn | grep |
| SET-8 | path-ctrl input readonly 背景 surface-2 | 静态读 |
| SET-9 | `<h3>控件设置</h3>` + 音量滑块 | grep |
| SET-10 | 滑块 160×6 圆角 6px | 静态读 |
| SET-11 | 滑块 thumb 18×18 圆 50% #fff box-shadow md | 静态读 |
| SET-12 | 滑块 thumb `left: calc(${vol}% - 9px)` | grep |
| SET-13 | 注脚小字 "设置项将在 Phase 3 接入真实系统设置" | grep |
| SET-14 | `.toggle` 44×26 圆角 pill | grep |
| SET-15 | `.toggle::after` 20×20 left 3 top 3 | 静态读 |
| SET-16 | `.toggle::after` 过渡 left var(--dur-base) var(--ease) | grep |
| SET-17 | `.toggle.on` 背景 primary + left 21px | grep |
| SET-18 | toggle role=switch tabindex=0 aria-checked | grep |
| SET-19 | Enter/Space 在 toggle 上触发 flip | 静态读 |
