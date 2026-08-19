# topbar-search

顶栏重做：搜索框改为 pill 圆角 + 关键词提示文字；排序器改为独立 pill 胶囊组。

## 范围

- `src/components/TopBar.vue` 视觉与文案调整
- 保留 Tauri 窗口控制按钮（最小化/最大化/关闭）

## 条款

### 搜索框

- MUST：搜索框背景 `var(--surface-2)`，border 1px `var(--border)`，圆角 `var(--r-pill)`，padding 8px 14px
- MUST：focus-within 时 border `var(--primary)` + box-shadow `0 0 0 3px var(--primary-soft)`
- MUST：内部含提示文字 `<span class="hint">大家都在搜：</span><span class="kw">极光</span><span class="hint">· 二次元 · 赛博城市</span>`（关键词文字 `var(--primary)` font-weight 600）
- MUST：搜索词仍通过 inject `topbarSearch` 共享 ref 驱动 grid 过滤

### 排序

- MUST：排序器外层背景 `var(--surface-2)`，圆角 `var(--r-pill)`，内 padding 3px
- MUST：2 个排序项（最热/最新）以 pill 形式横排；选中态 `background: var(--surface); color: var(--text); font-weight: 600; box-shadow: var(--sh-sm)`
- MUST：未选中态 `color: var(--text-2)`，hover `color: var(--text)`
- MUST：按下 `transform: scale(.94)`，过渡 150ms `var(--ease)`

### 窗口控制

- MUST：保留 TopBar 右侧 🌗 主题切换、— 最小化、▢ 最大化、✕ 关闭按钮
- MUST：theme 按钮切换 `data-theme` 在 `light/dark` 之间

## 验收

| ID | 条款 | 验证方式 |
|----|------|----------|
| TB-1 | 搜索框 `rounded-full` 或 `rounded-[var(--r-pill)]` | 静态读 |
| TB-2 | 搜索框 focus-within 有 primary border + primary-soft shadow | 静态读 |
| TB-3 | 搜索框内含"大家都在搜："+ "极光" 关键词 + "· 二次元 · 赛博城市" 提示 | grep |
| TB-4 | 排序器容器 `rounded-[var(--r-pill)]` 背景 surface-2 | 静态读 |
| TB-5 | 排序项选中态有 box-shadow: var(--sh-sm) | 静态读 |
| TB-6 | 排序项按下 scale(.94) 过渡 150ms | 静态读 |
| TB-7 | TopBar 含 主题切换 / 最小化 / 最大化 / 关闭 4 个按钮 | 静态读 |
| TB-8 | 主题按钮 emit 'theme'，App 处理切换 `light/dark`（非 frost-light/frost-dark） | 静态读 |
