# 灵镜文档索引

面向日常维护与迭代。Comet 阶段规格在 `docs/comet/`，本目录放**现行架构与目录说明**。

| 文档 | 说明 |
|------|------|
| [architecture.md](./architecture.md) | 三窗口架构、模块边界、数据流、进程模型 |
| [directory-structure.md](./directory-structure.md) | 仓库目录与关键文件职责 |
| [maintenance-notes.md](./maintenance-notes.md) | 已知坑、改动约束、排障清单 |
| [desktop-organize-reference.md](./desktop-organize-reference.md) | 桌面整理：图标映射、与腾讯方案对照、路线图 |
| [fence-ole-drag-out.md](./fence-ole-drag-out.md) | 分区拖出到 Explorer：OLE 方案、窗口判定、arm 生命周期、排障 |
| [comet/](./comet/) | 历史需求/规格/验收（OpenSpec 风格） |
| [comet/optimization-roadmap.md](./comet/optimization-roadmap.md) | 优化路线图 |
| [comet/specs/](./comet/specs/) | 能力规格（壁纸引擎、托盘、设置等） |

## 源码旁文档

| 路径 | 说明 |
|------|------|
| [`README.md`](../README.md) | 仓库入口：功能、开发命令、数据目录 |
| [`src-tauri/src/desktop_organize/README.md`](../src-tauri/src/desktop_organize/README.md) | 桌面整理 Rust 子模块拆分说明 |

## 建议阅读顺序（新人）

1. 根目录 `README.md`
2. `architecture.md`
3. `directory-structure.md`
4. 按任务读：壁纸 → `wallpaper/` + `public/wallpaper.html`；桌面整理 → `desktop-organize-reference.md` + `desktop_organize/`；拖出到资源管理器 → `fence-ole-drag-out.md`
5. 改动前扫一眼 `maintenance-notes.md`
