# favorites-page

我的收藏页：h3 标题 + "已收藏 N 张壁纸 · 本地保存，无需登录" 副文字 + grid 卡片列表。复用 WallpaperGrid 卡片样式。

## 范围

- `src/components/FavoritesView.vue` 新增
- 由 `App.vue` 在 `active === 'favorite'` 时显示
- 复用 `.grid` + `.card` 类（与 WallpaperGrid 一致）

## 条款

### 标题区

- MUST：`<h3 style="font-size:18px;font-weight:700;margin-bottom:4px;">我的收藏</h3>`
- MUST：副文字 font-size 12.5 color `var(--text-3)` margin-bottom 18，文字 "已收藏 ${count} 张壁纸 · 本地保存，无需登录"
- MUST：${count} 由 reactive computed 自 catalog 过滤 `favorite===true` 数

### 卡片列表

- MUST：复用 `.grid` 容器，3 列布局
- MUST：每张卡用与 WallpaperGrid 相同的 `.card .thumb .info .hover-acts` 结构
- MUST：仅渲染 `item.favorite === true` 的项
- MUST：空态：当 count === 0 时显示居中提示 "还没有收藏，去发现页点 ❤️ 收藏吧~"（color `var(--text-3)` font-size 13）
- MUST：每张卡可点击 → 触发与 WallpaperGrid 相同的 select 流程（抽屉滑入 + 倒计时）

### 导航联动

- MUST：App.vue 收到 `nav` 事件时根据 key 切换 main 区内容：
  - `online` → WallpaperGrid
  - `favorite` → FavoritesView
  - `settings` → SettingsView
  - `local` / `about` → 暂时占位（"该页面将在 Phase 2/3 接入"）
- MUST：切换时抽屉与当前 current 状态保留

## 验收

| ID | 条款 | 验证方式 |
|----|------|----------|
| FAV-1 | `src/components/FavoritesView.vue` 存在 | 静态读 |
| FAV-2 | h3 "我的收藏" font-size 18 font-weight 700 | grep |
| FAV-3 | 副文字 "已收藏 N 张壁纸 · 本地保存，无需登录" | grep |
| FAV-4 | N 由 computed 自 catalog 过滤 | 静态读 |
| FAV-5 | 卡片复用 `.card .thumb .info .hover-acts` 类 | grep |
| FAV-6 | 仅渲染 `favorite===true` 项 | 静态读 |
| FAV-7 | 空态文案存在 | grep |
| FAV-8 | App.vue 根据 active 切换 main 区 | 静态读 |
| FAV-9 | 卡片点击触发 select | 静态读 |
| FAV-10 | local/about 占位 "该页面将在 Phase 2/3 接入" | grep |
