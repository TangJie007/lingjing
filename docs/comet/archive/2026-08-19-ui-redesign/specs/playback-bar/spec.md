# playback-bar

底部播放条重做：46×32 缩略图 + 标题 + 4px 流动进度条 + 32×32 圆按钮（上一首/播放/下一首）+ 38×38 主色播放键 + 右侧 pill（音量/循环/导入，导入为 primary 色 pill）。

## 范围

- `src/components/PlaybackBar.vue` 全面重写
- 容器 `.playbar` flex items-center gap 14px padding 10 20 border-top 1px `var(--border)`，背景 `var(--surface-2)`
- `.pb-thumb` 46×32 圆角 6px box-shadow `var(--sh-sm)`
- `.pb-btns` flex gap 6px，3 个 `.pb-btn` 32×32 圆角 50%
- `.pb-now` flex 1 min-width 0，含 `.t` 标题 + `.bar` 4px 进度条 + `.bar i` 流动条
- `.pb-right` flex gap 8px，3 个 `.pill`

## 条款

### 容器与缩略图

- MUST：`.playbar` height 56px (h-14) flex-shrink-0 items-center gap-3 border-t border-[var(--border)] bg-[var(--surface-2)] px-5
- MUST：`.pb-thumb` width 46 height 32 圆角 6px box-shadow `var(--sh-sm)`
- MUST：缩略图 `background` 同步当前 current 的 thumb 渐变
- MUST：`.pb-now .t` font-size 13 font-weight 600 单行省略，文字"${title} · 正在播放"

### 进度条

- MUST：`.pb-now .bar` height 4px 圆角 4px 背景 `var(--border)` margin-top 6px overflow hidden
- MUST：`.pb-now .bar i` display block height 100% width 100% 背景 `var(--primary)` 圆角 4px transform-origin left
- MUST：`.pb-now .bar i` `@keyframes prog 18s linear infinite` `transform: scaleX(0)→scaleX(1)`
- MUST：`.pb-now .bar i.paused` `animation-play-state: paused`

### 控制按钮

- MUST：`.pb-btn` 32×32 圆角 50% 居中 border 1px `var(--border)` background `var(--surface)` color `var(--text-2)` font-size 13
- MUST：`.pb-btn:hover` `color: var(--text); border-color: var(--border-strong); transform: scale(1.08)`
- MUST：`.pb-btn:active` `transform: scale(.9)`
- MUST：`.pb-btn.play` width 38 height 38 background `var(--primary)` color #fff border-color `var(--primary)`
- MUST：上一首/播放/下一首文字 `⟨` / `▶ 或 ❚❚` / `⟩`

### 右侧 pill

- MUST：`.pb-right .pill` font-size 12 padding 6 12 圆角 pill border 1px `var(--border)` background `var(--surface)` color `var(--text-2)`
- MUST：pill hover `border-color: var(--border-strong); color: var(--text); transform: translateY(-1px)`
- MUST：pill 按下 `transform: scale(.94)`
- MUST：`.pb-right .pill.imp` color `var(--primary)` border-color `var(--primary-soft)` background `var(--primary-soft)`，文字"＋ 导入"

### 状态联动

- MUST：选中新卡片时，`.pb-thumb` 与 `.pb-now .t` 同步更新
- MUST：点击 `.pb-btn.play` 切换 ▶ ↔ ❚❚，同步切换 `.pb-now .bar i` 的 `paused` class
- MUST：cycleLoop 顺序 list → single → random，对应 🔁 / 🔂 / 🔀（保留 Phase 1 行为）
- MUST：未选 current 时显示 "未选择壁纸" / "—"

## 验收

| ID | 条款 | 验证方式 |
|----|------|----------|
| PB-1 | `.playbar` h-14 flex gap-3 border-t | 静态读 |
| PB-2 | `.pb-thumb` 46×32 圆角 6px | 静态读 |
| PB-3 | `.pb-now .bar` height 4px 圆角 4px 背景 border | grep |
| PB-4 | `@keyframes prog 18s linear infinite` 存在 | grep |
| PB-5 | scaleX(0)→scaleX(1) 流动条 | grep |
| PB-6 | `.pb-now .bar i.paused` animation-play-state paused | grep |
| PB-7 | `.pb-btn` 32×32 圆角 50% | 静态读 |
| PB-8 | `.pb-btn:hover` scale(1.08) | 静态读 |
| PB-9 | `.pb-btn.play` 38×38 主色 | 静态读 |
| PB-10 | 3 按钮文字 `⟨` / `▶ 或 ❚❚` / `⟩` | grep |
| PB-11 | `.pill` font-size 12 padding 6 12 圆角 pill | 静态读 |
| PB-12 | `.pill.imp` primary 色 pill | grep |
| PB-13 | 选中新卡片同步缩略图 + 标题 | 静态读 |
| PB-14 | 播放键切换同步 paused class | 静态读 |
| PB-15 | cycleLoop 顺序 list/single/random | 静态读 |
| PB-16 | 未选 current 时 "未选择壁纸" | 静态读 |
