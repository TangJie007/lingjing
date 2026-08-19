# motion-microinteractions

跨组件微动效与全局可达性：Toast 弹层、全局 `:focus-visible`、`prefers-reduced-motion`。

## 范围

- `src/components/Toast.vue` 新增（单例 ref + 全局触发）
- `src/composables/useToast.ts` 新增（trigger(msg) API）
- `src/styles/global.css` 补充：
  - `@keyframes fadeUp`
  - `@keyframes kenburns`
  - `@keyframes navRipple / navPop / gearSpin / heartBeat / pulse / countdown / prog`
  - `@keyframes toastIn`
  - `:focus-visible` 全局规则
  - `@media (prefers-reduced-motion: reduce)` 规则

## 条款

### Toast

- MUST：屏幕底部居中浮窗，`position: fixed; left: 50%; bottom: 36px; transform: translate(-50%, 20px)`
- MUST：背景 `var(--text)`，文字 `var(--bg)`，圆角 `var(--r-md)`，padding 12 20，box-shadow `var(--sh-win)`，font-size 13.5 font-weight 600
- MUST：进场 `transform: translate(-50%, 0); opacity: 1`，过渡 280ms `var(--ease)`
- MUST：离场反之，`opacity: 0`
- MUST：含 `.dot` 8×8 圆点 背景 `var(--success)`，左侧 8px 间距
- MUST：`role="status" aria-live="polite"`
- MUST：自动 2400ms 后消失（`setTimeout`）
- MUST：连续 trigger 时重置 timer
- MUST：trigger 由 `useToast` composable 暴露全局 `showToast(msg)` 函数
- MUST：使用方式：`showToast('壁纸已成功应用到桌面')`

### 触发点

- MUST：以下事件触发 Toast：
  - 抽屉"设为壁纸"按钮 → "壁纸「${title}」已成功应用到桌面"
  - 抽屉/卡片 ❤️ 切换 → "已收藏" / "已取消收藏"
  - 设置页"界面点击音效" toggle off → "音效已关闭"
  - 设置页"界面点击音效" toggle on → "音效已开启"
  - 卡片 hover-act "设为壁纸"（grid 上） → 同样 toast

### focus-visible

- MUST：`* :focus-visible` `outline: 2px solid var(--primary); outline-offset: 2px; border-radius: var(--r-sm)`
- MUST：仅键盘焦点触发，鼠标点击不显示
- MUST：覆盖所有可交互元素（button / input / toggle / nav-item / pill / chip / card）

### prefers-reduced-motion

- MUST：`@media (prefers-reduced-motion: reduce)` `*, *::before, *::after { animation: none !important; transition-duration: .01ms !important; }`
- MUST：覆盖全项目所有动画/过渡
- MUST：Ken Burns 在 reduce 下不启用
- MUST：fadeUp 入场在 reduce 下不启用
- MUST：progress 流动在 reduce 下不启用
- MUST：所有 stagger delay 失效

## 验收

| ID | 条款 | 验证方式 |
|----|------|----------|
| M-1 | `src/components/Toast.vue` 存在 | 静态读 |
| M-2 | `src/composables/useToast.ts` 存在 | 静态读 |
| M-3 | Toast 浮窗 bottom 36px transform translate(-50%, 20px) 起步 | grep |
| M-4 | Toast 进场 `transform: translate(-50%, 0); opacity: 1` 280ms | grep |
| M-5 | Toast 自动 2400ms 消失 | 静态读 |
| M-6 | Toast role=status aria-live=polite | grep |
| M-7 | 抽屉"设为壁纸"触发 Toast "壁纸已成功应用到桌面" | 静态读 |
| M-8 | ❤️ 切换触发 "已收藏 / 已取消收藏" Toast | 静态读 |
| M-9 | 音效 toggle 触发 Toast | 静态读 |
| M-10 | `*:focus-visible` 2px primary outline 2px offset | grep |
| M-11 | `@media (prefers-reduced-motion: reduce)` 全覆盖规则 | grep |
| M-12 | reduce 下 Ken Burns 不启用 | 静态读 |
| M-13 | reduce 下 fadeUp 不启用 | 静态读 |
| M-14 | reduce 下 progress 流动不启用 | 静态读 |
