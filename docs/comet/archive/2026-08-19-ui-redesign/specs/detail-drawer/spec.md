# detail-drawer

详情抽屉重做：保持 320px 宽 + 滑入；大图区 12s Ken Burns 缓慢推近；倒计时改为 50×50 圆形浮窗位于大图右下，中心显示大数字 + 副文字"秒后取消"，SVG 环 60s linear 归零；内容区改为 h3/by/meta/tags/ghost×3/主 CTA/注脚；收藏按钮态切换。

## 范围

- `src/components/DetailDrawer.vue` 全面重写
- 大图区 `.d-prev` aspect 16:10 + `.thumb-bg` + `.wheel.run` 50×50 圆形 + `.wheel-svg` 50×50 viewBox + `.wheel-num` 中心数字
- 内容区 `.d-body` 内含 h3 + by + meta + tags + act-row + btn-apply + d-note

## 条款

### 整体

- MUST：宽度 320px（`w-80`），border-left 1px `var(--border)`，背景 `var(--surface)`，从右滑入
- MUST：滑入用 `transform var(--dur-base) var(--ease), opacity var(--dur-base) var(--ease)`（沿用 A7 transform/opacity 铁律）
- MUST：抽屉关闭时 width 0 + opacity 0 + translateX(24px)，过渡 280ms

### 大图

- MUST：`.d-prev` aspect 16:10，position relative
- MUST：`.thumb-bg` 全覆盖 `position: absolute; inset: 0`
- MUST：`.detail.open .d-prev .thumb-bg` 触发 `@keyframes kenburns 12s var(--ease) infinite alternate` `transform: scale(1)→scale(1.12)`
- MUST：抽屉关闭时 Ken Burns 停止（class 移除）

### 倒计时

- MUST：`.wheel` 50×50 圆形，`position: absolute; right: 12px; bottom: 12px`，背景 `rgba(17,24,39,.35)`，`backdrop-filter: blur(4px)`，圆角 50%
- MUST：`.wheel-svg` 50×50 viewBox `0 0 50 50`，`transform: rotate(-90deg)`
- MUST：`.wheel-bg` stroke `rgba(255,255,255,.3)`，stroke-width 3，r=22 fill none
- MUST：`.wheel-ring` stroke `#fff`，stroke-width 3，stroke-linecap round，stroke-dasharray 138.23（= 2π·22），stroke-dashoffset 0 起步
- MUST：`.wheel.run .wheel-ring` `@keyframes countdown 60s linear forwards`，归零 `stroke-dashoffset: 138.23`
- MUST：`.wheel-num` 中心 flex column，文字 `#fff`
- MUST：`.wheel-num .n` font-size 18px font-weight 700 line-height 1，显示秒数
- MUST：`.wheel-num .l` font-size 9px opacity 0.85 margin-top 1px，文字"秒后取消"
- MUST：倒计时归零时抽屉自动关闭（沿用 A7 行为）
- MUST：点"取消"按钮停止定时器不关闭抽屉（沿用 A7 cancelAutoClose）

### 内容区

- MUST：`.d-body` padding 18px 20px flex 1 overflow auto
- MUST：h3 font-size 18px font-weight 700 显示壁纸标题
- MUST：`.by` font-size 12.5px color `var(--text-2)` margin-top 3px，文字 `by 设计者 · ${author}`
- MUST：`.d-meta` flex gap 18px margin 16px 0；每项 `.k` font-size 11px color `var(--text-3)` + `.v` font-size 13px font-weight 600
- MUST：meta 含 3 列：分辨率/大小/热度
- MUST：`.d-tags` flex flex-wrap gap 6px margin-bottom 16px
- MUST：`.act-row` flex gap 8px margin-bottom 12px
- MUST：`.btn-ghost` 3 个（收藏/分享/下载），flex 1 文字居中，padding 9px，border 1px `var(--border)`，圆角 `var(--r-md)`，background `var(--surface)`
- MUST：ghost hover `border-color: var(--border-strong); color: var(--text); transform: translateY(-1px)`
- MUST：ghost 按下 `transform: scale(.96)`
- MUST：`.btn-apply` width 100% padding 12px，background `var(--primary)`，文字 #fff font-size 14 font-weight 700，圆角 `var(--r-md)`，box-shadow `var(--sh-md)`
- MUST：apply hover `background: var(--primary-hover); transform: translateY(-1px) scale(1.01)`
- MUST：apply 按下 `transform: scale(.97)`
- MUST：`.d-note` font-size 11 color `var(--text-3)` margin-top 12 文字居中，文字"60 秒未操作将自动收起预览"

### 收藏态

- MUST：未收藏 ghost 灰色 + "♡ 收藏"
- MUST：收藏后 `.btn-ghost.liked` `color: var(--error); border-color: #FECACA; background: #FEF2F2`，文字 "♥ 收藏"
- MUST：深色下 `[data-theme="dark"] .btn-ghost.liked` `background: rgba(248,113,113,.12); border-color: rgba(248,113,113,.4)`

### 卡片 → 抽屉联动

- MUST：点击 grid 卡片后抽屉打开，缩略图渐变同步到大图 `.thumb-bg`
- MUST：卡片 selected 态保留直到抽屉关闭（与 G-4 一致）
- MUST：抽屉滑入同时启动 60s 倒计时
- MUST：滚动卡片 grid 不影响抽屉状态

## 验收

| ID | 条款 | 验证方式 |
|----|------|----------|
| D-1 | 抽屉 width 320 + border-left + 从右滑入 | 静态读 |
| D-2 | 滑入用 transform/opacity 过渡 280ms var(--ease) | grep |
| D-3 | `.d-prev` aspect 16:10 | 静态读 |
| D-4 | `@keyframes kenburns 12s var(--ease) infinite alternate` 存在 | grep |
| D-5 | `.detail.open .d-prev .thumb-bg` 触发 kenburns | 静态读 |
| D-6 | `.wheel` 50×50 圆形 right 12 bottom 12 backdrop-blur | grep |
| D-7 | `.wheel-svg` viewBox `0 0 50 50` rotate -90deg | grep |
| D-8 | stroke-dasharray 138.23 (=2π·22) 存在 | grep |
| D-9 | `@keyframes countdown 60s linear forwards` 存在 | grep |
| D-10 | `.wheel-num .n` 18px font-weight 700 显示秒数 | grep |
| D-11 | `.wheel-num .l` 9px "秒后取消" | grep |
| D-12 | 倒计时归零 emit('close') | 静态读 |
| D-13 | 取消按钮 cancelAutoClose 停 timer 不 emit close | 静态读 |
| D-14 | h3 + by + meta 3 列 + tags + ghost 3 个 + apply + d-note 结构 | 静态读 |
| D-15 | `.btn-ghost.liked` 红色 + 红边 + 浅红底 | grep |
| D-16 | `[data-theme="dark"] .btn-ghost.liked` rgba 浅红 | grep |
| D-17 | 卡片点击同步 thumb 渐变到大图 | 静态读 |
| D-18 | 卡片 selected 态与抽屉状态联动 | 静态读 |
