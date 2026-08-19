# wallpaper-engine

动态壁纸引擎（F4）：独立 Tauri worker 窗口经 Win32 Progman/WorkerW 置于桌面图标层之下；窗口内 WebView2 使用 HTML `<video>` / `<img>` 渲染；由主窗口通过 Tauri commands / events 控制。

## 范围

- `src-tauri`：创建/附着/销毁 wallpaper worker 窗口；Progman/WorkerW 附着；引擎命令（set / play / pause / volume / loop 协调所需状态）。
- worker 前端页（独立 HTML 或专用路由）：根据当前媒体 URI 与类型选择 `<video>` 或 `<img>`。
- 主窗口「设为壁纸」调用真实引擎，不再仅更新 `current` + toast。
- 多屏接口预留（枚举/目标 display id 可扩展），本阶段固定主显示器。

## 条款

### 窗口与附着

- MUST：应用在首次需要显示壁纸时创建（或复用）名为约定标识的 worker 窗口，无系统装饰、不抢焦点。
- MUST：worker 窗口通过 Progman/WorkerW 置于桌面图标层之下；失败时向主窗口返回可展示错误（toast 或状态），不得静默成功。
- MUST：退出应用时销毁或隐藏 worker，避免残留置顶/置底异常窗口。

### 播放后端

- MUST：worker 内使用 WebView2 文档播放，视频用 `<video playsinline>`，动画图/静图用 `<img>`（或等价）。
- MUST：支持媒体类型：MP4、WebM、GIF、WebP、常见静态图（至少 jpg/png/webp 静图）。
- MUST：不依赖 libmpv、外挂 MPV 进程或 WMF 自定义管线。

### 设为壁纸

- MUST：主窗口 `set_wallpaper`（或等价命令）传入可解析的本地/资源 URI + 元数据（id、title、type）。
- MUST：成功后桌面可见对应画面，主窗口 `current` 与播放条同步。
- MUST：发现页已绑定包内示例媒体的卡片设为壁纸可播；未绑定媒体的卡片 MUST toast「该资源暂无可用媒体」且不假装成功。

### 状态

- MUST：引擎向主窗口同步至少：playing、volume、muted、media id、可选 currentTime/duration。
- MUST：播放失败（文件缺失/解码失败）时回报错误，主窗口可见提示。

## 验收

| ID | 条款 | 验证方式 |
|----|------|----------|
| WE-1 | 存在 Progman/WorkerW 附着相关 Rust 实现 | 静态读 |
| WE-2 | worker 窗口创建/销毁路径存在 | 静态读 |
| WE-3 | worker 用 video/img 播放，无 mpv/WMF 依赖 | 静态读 + Cargo/package |
| WE-4 | set_wallpaper 成功后面面在桌面图标下可见 | 手动 |
| WE-5 | 无媒体绑定卡片 toast 且不成功附着错误片 | 手动/静态 |
| WE-6 | 播放失败有错误回报 | 静态读 |
