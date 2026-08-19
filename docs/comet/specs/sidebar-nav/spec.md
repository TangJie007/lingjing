# sidebar-nav

侧栏重做：5 个 SVG 线框图标 + 跨项滑动指示条 + 点击涟漪 + 弹簧回弹 + 专属动效 + Web Audio 合成音效 + 我的红点 badge。

## 范围

- `src/components/IconRail.vue` 全面重写
- `src/composables/useAudio.ts` 新增（Web Audio 合成）
- 5 个 SVG 图标：
  - 在线：4 个圆角矩形（grid）
  - 本地：文件夹
  - 我的：心形
  - 设置：齿轮
  - 关于：信息圆

## 条款

### 视觉

- MUST：5 个图标为 SVG 线框（stroke=currentColor，stroke-width 1.8，stroke-linecap/linejoin round），不再使用 emoji
- MUST：选中态背景 `var(--primary-soft)` + 文字 `var(--primary)` + 字重 600
- MUST：非选中态文字 `var(--text-2)`，hover 背景 `var(--surface-2)` + 文字 `var(--text)`
- MUST：3px 跨项滑动指示条，颜色 `var(--primary)`，圆角 `0 3px 3px 0`，通过 `--iy/--ih` CSS 变量驱动 `transform: translateY()`

### 动效

- MUST：点击触发动画组合（去重 reflow）：
  - 涟漪：`@keyframes navRipple 380ms var(--ease-out)`，位置由 `--rx/--ry` 控制
  - 弹簧回弹：`@keyframes navPop 280ms var(--ease-spring)` `scale(1)→.58→1.16→.95→1`
  - 设置图标：`@keyframes gearSpin .6s var(--ease)` `rotate(0→360°)`
  - 我的图标：`@keyframes heartBeat .72s var(--ease)` `scale(1→.8→1.2→.9→1.06→1`
- MUST："我的"右上 7×7 红点 `var(--error)`，持续 `@keyframes pulse 2s var(--ease) infinite` `scale(1)↔scale(1.35) opacity(1)↔.6`

### 音效

- MUST：启用"界面点击音效"时，点击播放 5 频率三角波（120ms 衰减）：
  - 在线 523.25 / 本地 587.33 / 我的 659.25 / 设置 783.99 / 关于 880.00 Hz
- MUST：首次调用时 `audioCtx = new AudioContext()` + 首次交互后 `audioCtx.resume()`
- MUST：设置 toggle 关闭时 `playClick` no-op
- MUST NOT：未引入任何音频素材文件

### 可达性

- MUST：每个 nav-item `role="button" tabindex="0" aria-label="..."`
- MUST：Enter/Space 触发 `fire`

## 验收

| ID | 条款 | 验证方式 |
|----|------|----------|
| S-1 | IconRail.vue 不再含 emoji 字符（仅 SVG） | grep 字符 |
| S-2 | 5 个 nav-item 各含 1 个 `<svg viewBox="0 0 24 24">` | 静态读 |
| S-3 | `.nav-indicator` class 存在且通过 CSS 变量 `--iy/--ih` 定位 | grep |
| S-4 | 选中态移动时 indicator 用 `transform: translateY()` 滑动（不是 left/top） | 静态读 |
| S-5 | `@keyframes navRipple 380ms var(--ease-out)` 存在 | grep |
| S-6 | `@keyframes navPop 280ms var(--ease-spring)` 存在 | grep |
| S-7 | `@keyframes gearSpin .6s var(--ease)` 存在 | grep |
| S-8 | `@keyframes heartBeat .72s var(--ease)` 存在 | grep |
| S-9 | `@keyframes pulse 2s var(--ease) infinite` 存在 | grep |
| S-10 | 我的项含 `.badge-dot` 元素 | grep |
| S-11 | `useAudio.ts` 含 `playClick(freq)` 函数 | grep |
| S-12 | `playClick` 用 `createOscillator` + `triangle` + 120ms 衰减 | 静态读 |
| S-13 | 5 频率常量 523.25/587.33/659.25/783.99/880.00 全部出现 | grep |
| S-14 | `audioCtx.resume()` 处理 suspended | 静态读 |
| S-15 | 音效 toggle 关闭时 `playClick` 立即 return | 静态读 |
| S-16 | 每个 nav-item 有 role=button + tabindex=0 + aria-label | 静态读 |
| S-17 | Enter/Space 在 nav-item 上触发 fire | 静态读 |
