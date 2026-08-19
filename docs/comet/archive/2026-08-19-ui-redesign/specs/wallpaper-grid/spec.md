# wallpaper-grid

中央卡片网格重做：固定 3 列 + 缩略图自身 hover 放大 + 卡片 hover translateY(-4px) + 选中态高亮 + LIVE badge + vol 标签 + 居中底部浮起快捷操作 + 错峰 fadeUp 入场。

## 范围

- `src/components/WallpaperGrid.vue` 全面重写
- grid 容器类名 `grid` + `repeat(3,1fr)` 固定 3 列
- 缩略图容器 `.thumb` aspect 16:10 + `.thumb-bg` 渐变 + `.badge` LIVE + `.vol` 大小 + `.hover-acts` 快捷操作
- 卡片信息区 `.info` 内含 `.t` 标题 + `.m` 副文（分类·分辨率）

## 条款

### 网格

- MUST：grid 固定 3 列 `grid-template-columns: repeat(3,1fr)`，间距 14px
- MUST：响应式 ≤680px 降为 2 列
- MUST：保留搜索/分类过滤逻辑（activeCat + topbarSearch）

### 卡片

- MUST：卡片整体 `transition: transform var(--dur-fast) var(--ease), box-shadow var(--dur-fast) var(--ease), border-color var(--dur-fast) var(--ease)`
- MUST：卡片 hover `transform: translateY(-4px); box-shadow: var(--sh-md); border-color: var(--border-strong)`
- MUST：卡片按下 `transform: translateY(-1px) scale(.99)`
- MUST：卡片选中态 `.selected` `border-color: var(--primary); box-shadow: 0 0 0 2px var(--primary-soft)`
- MUST：缩略图 `.thumb` 自身 hover 内部 `.thumb-bg` `transform: scale(1.07)`，过渡 280ms

### Badge

- MUST：缩略图左上 `.badge`（`type === 'video' || 'gif'` 时显示 "LIVE"），背景 `rgba(255,255,255,.9)`，文字 `#1F2329` font-weight 600，padding 2px 7px，圆角 pill
- MUST：缩略图右下 `.vol` 显示 `item.size`，背景 `rgba(17,24,39,.55)`，文字 `#fff`，padding 2px 7px，圆角 pill

### 快捷操作

- MUST：缩略图 hover 时 `.hover-acts` opacity 0→1，translateY(8px→0)，过渡 150ms
- MUST：内含 2 个 pill 按钮：
  - 预览：`.ha-btn.preview` `background: rgba(255,255,255,.94); color: #1F2329`，文字 "▶ 预览"
  - 应用：`.ha-btn.apply` `background: var(--primary); color: #fff`，文字 "设为壁纸"
- MUST：hover 按钮 `transform: scale(1.06)`，按下 `scale(.92)`

### 入场

- MUST：1-6 索引卡片按 60/120/180/240/300/360ms 错峰 `@keyframes fadeUp` `transform: translateY(14px)→0` + opacity 0→1，过渡 520ms `var(--ease-out)`
- MUST：`prefers-reduced-motion` 时全部 0.01ms

### 信息区

- MUST：`.info` padding 9px 11px；`.t` font-size 13px font-weight 600 单行省略；`.m` font-size 11px color `var(--text-3)` margin-top 2px，内容为 `${category} · ${res}`

## 验收

| ID | 条款 | 验证方式 |
|----|------|----------|
| G-1 | grid `grid-template-columns: repeat(3,1fr)` | grep |
| G-2 | 卡片 hover `translateY(-4px) + sh-md` | 静态读 |
| G-3 | 卡片按下 `translateY(-1px) scale(.99)` | 静态读 |
| G-4 | `.card.selected` 含 primary border + primary-soft 2px shadow | grep |
| G-5 | `.thumb-bg` hover `scale(1.07)` 过渡 280ms | 静态读 |
| G-6 | `.badge` 在 `type==video\|\|gif` 时显示 "LIVE" | 静态读 |
| G-7 | `.vol` 显示 `item.size` 在缩略图右下 | 静态读 |
| G-8 | `.hover-acts` opacity 0→1 + translateY(8→0) 过渡 150ms | 静态读 |
| G-9 | `.ha-btn.preview` 与 `.ha-btn.apply` 存在 | grep |
| G-10 | 入场 6 卡片 `.card:nth-child(1..6)` 60/120/180/240/300/360ms delay | grep |
| G-11 | 入场用 `@keyframes fadeUp` transform/opacity | grep |
| G-12 | `.info` 内 .t 标题与 .m 副文 存在 | grep |
| G-13 | 保留 activeCat 与 topbarSearch 过滤逻辑 | 静态读 |
| G-14 | 响应式 `@media (max-width: 680px)` 降 2 列 | grep |
