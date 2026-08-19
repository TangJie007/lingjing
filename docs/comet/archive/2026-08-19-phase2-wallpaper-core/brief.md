# Outcome

在已完成的 UI 壳（Phase 1 + `ui-redesign`）之上，落地可演示的动态壁纸闭环：真实壁纸引擎（F4）、本地导入与本地库（F3）、收藏持久化（F7）、播放条驱动真实引擎状态（F6）。用户「导入 → 设为壁纸 → 桌面动起来 → 播放条可控 → 收藏重启仍在」在 Windows 上可验收。

# Scope

整包 Phase 2（继承已归档 `mvp-implementation` D2）：

- **F4 壁纸引擎**：独立 Tauri worker 窗口，经 Win32 Progman/WorkerW 置于桌面图标层之下；worker 内用 WebView2 播放 HTML `<video>` / `<img>`；支持 MP4 / WebM / GIF / WebP / 静态图；「设为壁纸」从 toast 占位变为真实应用。
- **F3 本地导入 / 本地库**：侧栏「本地」页替换占位，仅网格视图（复用现有卡片样式）；支持文件选择器 + 主窗口拖放导入；导入项进入应用数据目录下的本地库并可设为壁纸；播放条「导入」打开同一导入入口。
- **F7 收藏持久化**：收藏与本地库索引写入应用数据目录 JSON；重启后「我的收藏」与 hint「已收藏 N 张壁纸 · 本地保存，无需登录」仍正确。
- **F6 播放条驱动**：播放/暂停、上一首/下一首、音量（含静音）、循环模式（列表/单曲/随机）与引擎状态同步；有时长媒体用真实进度，无时长（如部分 GIF/静图）用明确代理态；缩略图点击回详情抽屉。
- **在线演示资源**：为发现页前若干条 catalog 打包示例媒体，点「设为壁纸」即可播真实画面。

变更形态：单一 Native change。

# Non-goals

- Web/HTML5 壁纸页、Shader/GLSL、3D 壁纸（V1+）。
- libmpv / 外挂 MPV / Windows Media Foundation 原生解码路径。
- 多屏分配功能（仅保留接口钩子，不实现 Span/Per-display）。
- 在线资源库后端、下载、登录、云同步。
- 文件夹批量导入；本地库列表双视图与类型/大小/时间筛选（留后续）。
- Phase 3：性能/电源自动暂停（F9）；设置中心系统项真接入（开机启动 / 双击隐藏图标 / 全屏变静态）；设置页「更改壁纸路径」与迁移工具。
- macOS / Linux。
- 改动 `ui-redesign` 已通过的视觉 Token / 动效铁律（仅接线、补本地库页、播放条控件功能化）。

# Acceptance examples

| ID | 内容 |
|----|------|
| A1 | 对带示例媒体的在线卡片或已导入本地项执行「设为壁纸」后，桌面图标层下方出现对应动态/静态画面（不仅 toast） |
| A2 | 通过文件选择器或拖放导入 MP4/WebM/GIF/WebP/常见静态图后，项出现在「本地」网格，并可设为壁纸 |
| A3 | 播放条播放/暂停与桌面壁纸播放状态一致；暂停后动态画面静止 |
| A4 | 上一首/下一首在当前可播放队列内切换，桌面画面与播放条标题/缩略图同步 |
| A5 | 音量滑块与静音影响有音轨壁纸的音量；无音轨时操作不报错 |
| A6 | 循环模式按 列表→单曲→随机 切换，行为可观察（列表尾切下一项 / 单曲重播 / 随机下一项） |
| A7 | 切换收藏后完全退出并重启应用，「我的收藏」仍显示对应项且 N 正确 |
| A8 | 播放条「＋ 导入」与本地库页导入入口触发同一导入流程且均可完成一次导入 |
| A9 | 发现页至少前 6 条（或约定映射的）占位卡片绑定包内示例媒体，设为壁纸可播 |
| A10 | `pnpm build` 通过；`pnpm tauri build`（或项目约定的 Windows 桌面构建）通过 |

# Constraints and invariants

- 主窗口 = UI 控制层；worker 窗口 = 桌面壁纸渲染层（Progman/WorkerW）；前端仅经 Tauri commands / events 控制引擎。
- 离线优先、零登录。
- 动效铁律：仅动画 `transform` / `opacity`。
- 主题继续 token 化。
- 导入文件复制/登记到应用数据目录（如 `library/`）；设置页路径控件本阶段仍可为只读占位。
- 持久化使用应用数据目录 JSON（收藏 ID/状态 + 本地库索引），不引入 `tauri-plugin-store`，不用 localStorage 作为唯一真源。
- Windows only。

# Decisions

- D0（范围）：整包 Phase 2 = F4 + F3 + F7 + F6；隔离 `current`（`v1.1.0`）；语言 zh-CN；单一 Native change。
- D1（架构，继承）：dedicated Tauri worker 窗口 + Win32 Progman/WorkerW；Vue 主窗为控制层。
- D3（播放后端）：worker 内 **WebView2 + HTML `<video>` / `<img>`**，不引入 libmpv/WMF。
- D4（在线设壁纸）：为发现页 **前若干条（≥6）打包示例媒体**，点「设为壁纸」播放对应资源；无绑定媒体的卡片 toast 说明「该资源暂无可用媒体」。
- D5（导入）：**文件选择器 + 主窗口拖放**；不做文件夹批量。
- D6（格式）：**MP4 / WebM / GIF / WebP / 常见静态图**（jpg/png/webp 静图等）。
- D7（本地库 UI）：**仅网格**，复用现有卡片视觉；不做列表双视图与高级筛选。
- D8（持久化）：应用数据目录 **JSON 文件**（Tauri FS）存收藏与本地库索引。
- D9（库根路径）：导入落地应用数据目录；设置「更改路径」留 Phase 3。
- D10（进度）：有 `duration` 的媒体用真实 `currentTime/duration` 驱动进度；无时长媒体显示满格或脉冲代理态，且暂停时静止。

# Open questions

（无。用户已于 Shape 确认 Outcome / Scope / Non-goals / Decisions D0–D10 / Acceptance A1–A10。）

# Verification expectations

- 构建：`pnpm build` 干净通过；Windows 桌面构建按约定命令通过。
- 手动：示例卡设壁纸可见桌面画面；导入真实文件 → 本地库 → 设壁纸；播放条 play/pause/prev/next/volume/loop；重启后收藏仍在。
- 静态：存在 worker 窗口创建与 Progman/WorkerW 附着相关 Rust；前端 `invoke`/事件覆盖 set/play/pause/seek-state/volume/import/favorites/library。
- 回归：`ui-redesign` 视觉与侧栏/发现/设置/收藏页不回退；「本地」不再显示 Phase 2/3 占位文案。
