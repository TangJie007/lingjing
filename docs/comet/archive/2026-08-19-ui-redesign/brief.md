# 灵镜 UI/UX 重设计交付

## 概要

依据 `lingjing-ui-design.html` 设计交付稿（"雾银极简 / 雾银暗夜"主题，含完整 Token 体系、可点击交互原型与微动效），对 MVP 现有 UI/UX 做一次全面重设计，使其与已通过设计评审的视觉/交互基线一致，并补齐"设置中心"与"我的收藏"两个新页面。

## 背景

`mvp-implementation` 变更（已归档）完成了 PRD F1–F9 的骨架落地（21/21 acceptance items 全部通过），但实现层在视觉、交互、动效、命名上与设计交付稿存在显著偏差：

- 命名：实现用"灵境 LingScape"，设计稿用"灵镜 LINGJING"
- 侧栏图标：实现用 emoji 字符，设计稿规定使用 SVG 线框图标
- 侧栏指示：实现每项内嵌 4×24 块，设计稿规定一条跨项 3px 滑动指示条
- 卡片：实现网格自适应列数 + 缩略图位于卡片底部滑入 hover 条；设计稿规定固定 3 列 + 缩略图自身 hover 放大 + 居中底部浮起的快捷操作
- 详情抽屉：实现 60×60 SVG 嵌在文字旁 + 副文字"预览将在 N s 后自动收起"；设计稿规定 50×50 圆形浮窗位于缩略图右下，中心显示大数字 + 12s Ken Burns 大图
- 播放条：实现矩形按钮 + 无进度条；设计稿规定 32×32 圆按钮 + 38×38 主色播放键 + 4px 流动进度条
- 缺失：Toast 弹层、Web Audio 点击音效、Ken Burns 大图、设置页、收藏页

## 期望成果

应用 `灵镜 LINGJING` 在视觉、交互、动效层面与设计交付稿一致；新增设置页与我的收藏页覆盖 F8、F7 全部 UI 流；Theme/Sound 切换、动效 Token、卡片入场、抽屉倒计时等基础能力在桌面端（Windows）能跑通。

## 范围

### In-Scope

- **品牌重塑**：应用名 "灵镜 LINGJING" 覆盖到窗口标题、TopBar 文案、CLI 命名（仅 UI 层面，不强制改 npm 包名/Tauri identifier）
- **设计 Token 升级**：补齐 `global.css` 至与设计稿 1:1（含 light/dark 双套、surface-2/border-strong/text-2/text-3/primary-hover/ease-out/--s1..s12/--r-sm..xl/--sh-sm|md|win 等），并将 Tailwind v4 的 `@theme` 注册与原生 CSS 变量统一
- **侧栏重做**：
  - 5 个 SVG 线框图标（在线/本地/我的/设置/关于）
  - 跨项 3px 主色滑动指示条（用 `--iy/--ih` CSS 变量驱动）
  - 点击涟漪（`@keyframes navRipple`，380ms `--ease-out`）
  - 图标弹簧回弹（`@keyframes navPop`，280ms `--ease-spring`）
  - "我的"图标❤️心跳（`@keyframes heartBeat`，720ms）
  - "设置"图标齿轮转 360°（`@keyframes gearSpin`，600ms）
  - "我的"项右上 7×7 红点 badge 持续 pulse（`@keyframes pulse`，2s 循环）
  - **Web Audio 合成点击音效**（5 频率：在线 523.25 / 本地 587.33 / 我的 659.25 / 设置 783.99 / 关于 880.00 Hz，三角波 120ms 衰减，由设置项"界面点击音效"开关控制）
- **顶栏重做**：搜索框改为 pill 圆角 + 关键词提示文字 "大家都在搜：极光·二次元·赛博城市"；排序器改为独立 pill 胶囊组
- **分类标签**：保持现有 9 类别，但视觉改为 pill 边框、hover translateY(-1px)、按下 scale(.94)
- **卡片重做**：
  - 网格改为 `repeat(3,1fr)` 固定 3 列
  - 缩略图 hover 自身 `scale(1.07)`（280ms）
  - 卡片 hover `translateY(-4px) + sh-md`、按下 `translateY(-1px) scale(.99)`
  - 卡片选中态 `border-color: primary + box-shadow: 0 0 0 2px primary-soft`
  - 缩略图左上 LIVE badge（动/视频）/ 右下大小标签
  - 缩略图 hover 居中底部浮起快捷操作（▶ 预览 / 设为壁纸），opacity + translateY(8px→0) 过渡
  - 卡片按 1/2/3/4/5/6 索引错峰 60/120/180/240/300/360ms `fadeUp` 入场（沿用 A19 的 transform/opacity 铁律）
- **详情抽屉重做**：
  - 宽度仍 320px，从右滑入（保持现有 transition 形态）
  - 大图区域：16:10 + 12s Ken Burns `scale(1)→scale(1.12)` 缓慢推近（仅在 detail.open 时启用）
  - **倒计时改形态**：50×50 圆形浮窗位于大图右下，半透明深色底 + `backdrop-filter: blur(4px)`，中心显示大数字（18px font-weight 700）+ 副文字"秒后取消"（9px 0.85 opacity），SVG 圆环用 `stroke-dasharray: 138.23` + `stroke-dashoffset` 60s linear 归零
  - 内容区：h3 标题 + by 作者 + meta 行（分辨率/大小/热度） + tags + ghost 收藏/分享/下载 3 个 + 主 CTA"设为壁纸" + 注脚 "60 秒未操作将自动收起预览"
  - 收藏按钮态：未收藏 ghost 边框 + 灰色文字"♡ 收藏"；收藏后 `color: error + border-color: #FECACA + background: #FEF2F2`，文字变 "♥ 收藏"
- **播放条重做**：
  - 缩略图 46×32 + 标题 "极光幻境 Aurora · 正在播放"
  - 4px 进度条（`scaleX(0)→scaleX(1)` 18s 循环，播放时跑、暂停时 `animation-play-state: paused`）
  - 32×32 圆按钮（上一首 `⟨` / 播放 `▶/❚❚` / 下一首 `⟩`），38×38 主色播放键
  - 右侧 pill 按钮：🔊 音量 / 循环 / ＋ 导入（导入用 primary 色 pill）
- **主题切换**：
  - data-theme 命名从 `frost-light/frost-dark` 改为 `light/dark`
  - 浅色/深色 token 与设计稿 1:1
  - 切换走 `var(--dur-base) var(--ease)` 平滑过渡
- **设置页**（PRD F8 UI 侧）：
  - 基本设置：4 个 toggle（开机启动动态壁纸 / 鼠标双击隐藏桌面图标 / 其他程序全屏时变为静态 / 界面点击音效）
  - 壁纸路径：readonly input + 更改路径按钮
  - 控件设置：默认播放音量滑块（160×6 圆角 + 18×18 圆形 thumb）
  - toggle 44×26 圆角 + 20×20 圆形滑块 + 280ms left 过渡
- **我的收藏页**（PRD F7 UI 侧）：
  - h3 标题"我的收藏"
  - 副文字"已收藏 N 张壁纸 · 本地保存，无需登录"
  - 与主页相同的 grid 卡片列表（仅显示 favorite=true 项）
- **Toast 弹层**：屏幕底部居中浮窗，离屏 transform(translateY(20px)) + opacity 0，进场 transform(0) + opacity 1，自动 2400ms 消失；用于"壁纸已成功应用到桌面""收藏成功""音效关闭"等反馈
- **a11y**：
  - 全局 `:focus-visible { outline: 2px solid var(--primary); outline-offset: 2px; }`
  - 所有可交互元素加 `role` + `aria-label` + `tabindex="0"`
  - `prefers-reduced-motion` 把 transition/animation 降到 0.01ms
- **a11y / 设置**：音效 toggle 默认 `on`；当 toggle 关掉时 `playClick` 立即 no-op
- **构建验证**：`pnpm build` 干净通过，vue-tsc + vite 0 错误；35+ modules 打包 CSS/JS 体积在合理区间

## 非目标

- 不做真实的 wallpaper engine 启动逻辑（仍维持 "设为壁纸" 仅更新前端 current ref 的 Phase 1 行为）
- 不做真实的多屏分配（PRD F5），仅 UI 占位
- 不做真实文件导入流程，导入按钮仅触发 toast 提示
- 不做云端收藏同步、登录、账户系统
- 不做 macOS/Linux 适配（仅 Windows 验证）
- 不动 PRD F9（性能/电源自动暂停）的实现
- 不改 tauri.conf.json 的 identifier、productName（仅 UI 文案 / 窗口标题）
- 不引入新的 npm 依赖（如不需要），已锁定的依赖（vue、tauri、tailwindcss v4）继续使用

## 验收示例

| ID | 内容 |
|----|------|
| A1 | 应用启动后窗口标题和顶栏 Logo 文字均为"灵镜 LINGJING"，不再出现"灵境"字样 |
| A2 | 顶栏搜索框为 pill 圆角，placeholder 写"大家都在搜：极光·二次元·赛博城市"（纯展示） |
| A3 | 侧栏 5 个图标为 SVG 线框（在线/本地/我的/设置/关于），不再使用 emoji |
| A4 | 点击侧栏任一图标，3px 主色滑动指示条通过 `transform: translateY()` 滑到对应项（用 `--iy/--ih` CSS 变量驱动） |
| A5 | 点击侧栏图标时，触发动画：涟漪扩散（380ms）+ 图标弹簧回弹（280ms） |
| A6 | 点击"设置"图标，触发 360° 齿轮旋转（600ms ease） |
| A7 | 点击"我的"图标，触发❤️心跳动画（720ms） |
| A8 | "我的"项右上 7×7 红点 badge 持续 pulse（2s 循环） |
| A9 | 启用"界面点击音效"后，点击侧栏图标播放对应频率三角波（120ms 衰减）；关闭 toggle 后无声音 |
| A10 | 卡片网格固定 3 列；卡片缩略图 hover 自身 `scale(1.07)` |
| A11 | 卡片 hover `translateY(-4px) + sh-md`；按下 `translateY(-1px) scale(.99)` |
| A12 | 卡片缩略图左上 LIVE badge（type==video|gif 时显示）、右下大小标签 |
| A13 | 卡片缩略图 hover 时，居中底部浮起快捷操作（▶ 预览 / 设为壁纸） |
| A14 | 选中卡片有 `border-color: primary + box-shadow: 0 0 0 2px primary-soft` 高亮 |
| A15 | 卡片按 1/2/3/4/5/6 索引错峰 60/120/180/240/300/360ms `fadeUp` 入场 |
| A16 | 详情抽屉大图 12s Ken Burns `scale(1)→scale(1.12)` 缓慢推近（仅 detail.open 时启用） |
| A17 | 详情抽屉倒计时为 50×50 圆形浮窗，位于大图右下，中心显示大数字 + 副文字"秒后取消" |
| A18 | 详情抽屉内容：h3 标题 + by 作者 + meta 行 + tags + ghost 3 个 + 主 CTA"设为壁纸" + 注脚 |
| A19 | 收藏按钮态：未收藏为 ghost 边框 + 灰色"♡ 收藏"；收藏后变红色 + 红边 + 浅红底"♥ 收藏" |
| A20 | 播放条 4px 流动进度条 18s 循环；暂停时 `animation-play-state: paused` |
| A21 | 播放条 32×32 圆按钮（上一首/播放/下一首）+ 38×38 主色播放键 + 右侧 pill 按钮（音量/循环/导入） |
| A22 | 主题切换：data-theme 命名 light/dark；浅色/深色 token 与设计稿 1:1；切换走 280ms 过渡 |
| A23 | 设置页：基本设置 4 个 toggle（开机启动/双击隐藏/全屏静态/界面音效）+ 路径控件 + 音量滑块 |
| A24 | toggle 44×26 圆角 + 20×20 圆形滑块 + 280ms left 过渡 |
| A25 | 我的收藏页：h3 标题 + "已收藏 N 张壁纸 · 本地保存，无需登录" 副文字 + grid 卡片列表 |
| A26 | Toast 弹层：底部居中浮窗，进场 transform/opacity、自动 2400ms 消失 |
| A27 | 全局 `:focus-visible` 2px primary outline |
| A28 | `prefers-reduced-motion: reduce` 把 transition/animation 降到 0.01ms |
| A29 | 切换主题、走完一个完整卡片→抽屉→播放条交互后，`pnpm build` 干净通过（vue-tsc + vite build exit 0） |

## 约束与不变量

- 动效铁律：仅动画 `transform` / `opacity`（GPU 友好，不触发重排），保持 Phase 1 已通过的 A19-A21 约束
- 主题 token 化：所有颜色/间距/圆角/阴影必须走 `var(--xxx)`，禁止硬编码
- 离线优先、零登录
- Tauri 窗口 chrome 仍可拖（header `app-region: drag`），交互控件 `app-region: no-drag`
- 复用 `mvp-implementation` 已建的数据层（`src/data/catalog.ts`），不重写
- 性能预算：CSS 增量 < 30kB、JS 增量 < 50kB（仍由 Vite 默认代码分割控制）

## 决策

- D1（命名）：**应用名统一为"灵镜 LINGJING"**，包括窗口标题（`tauri.conf.json.windows[0].title`）、顶栏 Logo（"灵镜 LINGJING" + 副文"动态壁纸"）、文档/CLI 命名。仅 UI 层面改动；Tauri bundle identifier（`com.lingscape.app`）与 npm 包名（`ling-scape`）保持不变（避免破坏已发布的桌面安装）。
- D2（实施方式）：**单一 Native change 一次性重做 UI**，不拆 Supervisor Change。理由：6 个 spec（shell/discover/detail/playback/settings/favorites/motion）共享同一套 token、同一套动效铁律、同一批组件，拆分会引入跨 worktree 集成成本，得不偿失。
- D3（音效）：**Web Audio 合成**，不引入音频素材文件。3 个频率表（在线/本地/我的/设置/关于）已固定，无需用户配置。
- D4（dark theme 命名）：使用 `data-theme="light/dark"`，**不再使用** `frost-light/frost-dark`。理由：设计稿已固定这两个 key，命名差异会让 future 维护者困惑。
- D5（Toast）：**内置底部居中浮窗**，单实例 ref 管理；不引入 Toast 库。触发点："壁纸已成功应用到桌面""收藏成功 / 取消""音效关闭"等。
- D6（Ken Burns）：**仅在大图区启用**（detail.open 时 class 触发），关闭抽屉时停止动画。`prefers-reduced-motion` 时不启用。
- D7（保留能力）：**保留 Phase 1 已通过的所有 A1–A21 acceptance items** 的功能语义（三区布局、Frost Light 浅色 token、动效 tokens、占位 catalog 浏览、60s 倒计时、右键菜单、键盘可达、Tauri 窗口拖动）。本变更只"换皮 + 补缺"，不重做底层能力。
- D8（数据层）：**不动 `src/data/catalog.ts`**，仍使用 24 个占位条目 + 8 套渐变。Settings/Favorites 所需字段（如路径、收藏列表）从内存 state 起步，**不接持久化**（持久化留给 Phase 3 变更）。
- D9（设置页 toggle 状态）：**仅 UI 占位**，不真的改系统设置。所有 toggle 状态只存在内存 ref，刷新后重置。Phase 3 时接入 Tauri commands。

## 开放问题

待 Shape 确认时与用户对齐；本变更启动前必须关闭。

- [ ] Q1：dark theme 在 Tauri 启动时是默认 light 还是跟随系统？设计稿假设默认 light（`<html lang="zh-CN" data-theme="light">`），建议**默认 light + 手动切换**（与 Phase 1 一致，不接 `prefers-color-scheme`）。
- [ ] Q2：音效的"开/关"初始值是？设计稿设置页 toggle 默认 on。建议**默认 on**（首次启动有反馈），用户可在设置页关闭。
- [ ] Q3：favorite 切换在 Phase 1 行为上是 toggle `it.favorite = !it.favorite` 但仅改内存；本变更**保持同语义**（不接持久化），通过？我的收藏页可正确显示？预期：**保持内存语义**。
- [ ] Q4：设置页 toggle 切换后是否需要即时反馈？例如切换"界面点击音效"后立刻播/静音一次？设计稿无明确指示。建议**仅影响后续点击**，不在 toggle 自身上播。
- [ ] Q5：Web Audio 合成需要在用户首次交互后 `audioCtx.resume()`（浏览器自动策略），不需用户额外确认？预期：**是**，通过 `if (audioCtx.state === 'suspended') audioCtx.resume();` 自动处理。
- [ ] Q6：是否保留 Phase 1 的 `WallpaperList.vue` / `PreviewStage.vue` 死代码？建议**在本次变更中删除**（verification 报告已经标为 follow-up 清理项）。

## 验证期望

- **构建**：`pnpm build` 干净通过（vue-tsc + vite build exit 0）
- **静态扫描**：grep 不得出现 `style="color:#"` 之类的硬编码颜色，所有颜色必须 `var(--xxx)` 或 token 别名
- **手动验收**：
  1. 启动 Tauri dev，确认窗口标题为"灵镜 LINGJING"
  2. 默认进入 light 主题，顶栏 / 卡片 / 抽屉 / 播放条 视觉与设计稿 §2 一致
  3. 点击侧栏"设置"图标，齿轮转 360°；点击"我的"，❤️ 心跳
  4. 点击"在线"听到 523Hz 三角波；切到"关于"听到 880Hz
  5. 切到 dark 主题，token 平滑切换；切回 light
  6. 打开任一卡片：缩略图选中态高亮、抽屉滑入、大图缓慢推近、右下 50×50 圆形倒计时数字从 60 走到 0、归零自动收起
  7. 点"取消"，倒计时冻结不收起
  8. 在设置页关掉音效，再点侧栏图标无声音；开启后恢复
  9. 切到"我的收藏"页，看到 favorite=true 的卡片；点❤️可移除（仅内存）
  10. 把系统切到 `prefers-reduced-motion: reduce`，所有 transition/animation 立即 0.01ms
- **可达性**：
  - 键盘 Tab 在所有可交互元素间循环
  - `:focus-visible` 2px primary outline 可见
  - 倒计时 SVG 环 + 数字对屏幕阅读器有可读 label

## 关联文件

- 设计交付稿：`lingjing-ui-design.html`（项目根）
- 已归档的 Phase 1 骨架：`docs/comet/archive/2026-08-19-mvp-implementation/`
- 现有组件：
  - `src/App.vue`
  - `src/components/IconRail.vue` ← 大改
  - `src/components/TopBar.vue` ← 小改
  - `src/components/WallpaperGrid.vue` ← 大改
  - `src/components/DetailDrawer.vue` ← 大改
  - `src/components/PlaybackBar.vue` ← 大改
  - `src/components/WallpaperList.vue` ← 删除（死代码）
  - `src/components/PreviewStage.vue` ← 删除（死代码）
  - `src/components/SettingsView.vue` ← 新增
  - `src/components/FavoritesView.vue` ← 新增
  - `src/components/Toast.vue` ← 新增
  - `src/composables/useAudio.ts` ← 新增（Web Audio 合成）
- 现有样式：
  - `src/styles/global.css` ← 大改（补齐 token、Ken Burns、ripple、pop、spin、beat、pulse、toast、focus-visible、prefers-reduced-motion）
- 现有数据：
  - `src/data/catalog.ts` ← 不动
- Tauri：
  - `src-tauri/tauri.conf.json` ← windows[0].title 改为 "灵镜 LINGJING"
  - `src-tauri/src/lib.rs` ← 不动

## 风险

- 改动量大（30+ acceptance items、6-7 个组件、1 份大 CSS、2 个新页面、1 个 composable），单次 Build → Verify 容易踩到回归；通过把 acceptance items 拆细（A1-A29）让 Verifier 颗粒度可控
- Web Audio 在 Electron 早期版本默认禁用 autoplay；Tauri 2.x WebView 走 WebView2 / WebKit，行为与浏览器一致；用户首次点击后 `audioCtx.resume()` 是稳定的 fallback
- dark theme 的 `--primary: #818CF8` 比 light 更亮，可能与现有 Phase 1 `frost-dark` 不一致（Phase 1 没设深色 token），需要在 build 时单独验证深色截图
- Settings 4 个 toggle 仅 UI 占位，用户预期"切换后立即生效"会落空；需在设置页底部加一行小字 "设置项将在 Phase 3 接入" 提示
