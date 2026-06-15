# 技术设计：灵境（LingScape）V2.0

**关联 Spec**：`specs/` 下全部 8 模块 36 功能（V2 新增 15 + 修改 1）
**PRD 版本**：AI动态桌面_V2_PRD.md v2.0
**日期**：2026-06-15
**技术栈**：Tauri (Rust) + Vue 3 + Vite（延续 V1）

## 1. 目标与范围

### 做什么

- 壁纸库全面视觉化升级：卡片缩略图网格、动态预览、多维度排序、收藏标签搜索
- 壁纸详情页：大图预览 + 完整元数据 + AI 方案信息 + 动态播放控制
- 退出恢复原始桌面：快照备份 + 正常退出恢复 + 崩溃恢复检测
- 完整桌面分区整理：对标 Fences，视觉覆盖式实现（创建/命名/调色/卷起/图标拖入）
- 多平台 API 扩展：DeepSeek（第一步）+ 可灵（第二步视频）
- AI 体验升级：分步进度反馈 + 历史方案重新生成
- 定时轮换、多显示器独立壁纸、快捷键、播放调节、应用内更新检测

### 不做什么

- 不做 macOS / Linux（V3.0+）
- 不做 AI 实时壁纸演变（V3.0）
- 不做 UGC 社区 / 创意工坊（V3.0）
- 不做任务栏美化、桌面小组件（评估中）
- 不做替换系统桌面（自己渲染图标）

## 2. 架构决策

| 决策 | 选择 | 理由 |
|------|------|------|
| 缩略图提取 | **FFmpeg**（MP4/WebM） + 内置（GIF 第一帧） | FFmpeg 功能强，支持所有视频格式；GIF 用 Rust image crate 直接读第一帧 |
| 缩略图缓存 | WebP 格式，`{hash}.thumb.webp`，存壁纸库目录 | WebP 体积小、质量好；hash 命名避免冲突 |
| 桌面整理实现 | **视觉覆盖式**（路线A）：透明 Win32 窗口覆盖，图标实际位置不动 | 不破坏系统桌面，Explorer 重启后易恢复，兼容性好 |
| 分区容器 | `WS_POPUP + WS_EX_LAYERED + WS_EX_TOOLWINDOW`，Z-order 设为桌面层级 | 每个分区独立透明窗口，不干扰图标交互 |
| 图标位置记录 | 本地 JSON：`{iconPath, partitionId, gridX, gridY}` | 图标实际位置不动，灵境记录逻辑归属 |
| 退出恢复 | 启动时备份壁纸路径到 `%AppData%/LingScape/backup/`，退出时 `SPI_SETDESKWALLPAPER` 恢复 | 标准 Windows API，不依赖第三方 |
| 定时轮换 | Rust 后台定时器 + `set_wallpaper` 命令 | 复用现有壁纸引擎，不引入新依赖 |
| 多显示器 | 复用现有 MPV 多屏架构，每屏独立 `currentWallpaperId` | V1 已支持多屏 MPV，扩展状态管理即可 |
| 更新检测 | 启动时 HTTP GET GitHub Release API / 自建 JSON | 轻量，不依赖第三方更新框架 |
| 快捷键 | Tauri global shortcut plugin 或 Win32 `RegisterHotKey` | 全局快捷键需系统级注册 |
| 播放调节 | MPV `--speed` / `--brightness` / `--saturation` / `--contrast` 属性 | MPV 原生支持，通过 IPC 设置 |
| 生成进度 | 前端分步状态机 + 预估时间（基于历史平均） | 不依赖 API 实时进度回调 |

## 2.1 设计准则（继承 V1，全部强制遵守）

| # | 准则 | 说明 |
|---|------|------|
| 1 | **单一职责** | 每个模块/文件/函数只做一件事 |
| 2 | **不过度设计** | V2 只做 Spec 明确要求的功能 |
| 3 | **文件行数上限 500 行** | 超过时拆分 |
| 4 | **优先第三方库** | 满足维护标准的库优先使用 |
| 5 | **关键代码必须注释** | Rust 用英文，Vue/TS 用中文 |

### 2.1.1 V2 新增第三方库白名单

| 类别 | 库名 | 版本 | 用途 |
|------|------|------|------|
| Rust 图像 | `image` | latest stable | GIF 第一帧提取、缩略图处理 |
| Rust WebP | `webp` | latest stable | 缩略图 WebP 编码 |
| Rust 快捷键 | `tauri-plugin-global-shortcut` | latest stable | 全局快捷键注册 |
| 视频缩略图 | **FFmpeg**（外部可执行文件） | 系统安装或打包 | MP4/WebM 缩略图提取（`ffmpeg -ss 0.5 -i in.mp4 -vframes 1 out.webp`） |

## 3. 数据流

### 3.1 壁纸库视觉化数据流

```
用户打开壁纸库
  → LibraryPage.vue 加载 wallpapers[]
  → 对每个 video/gif 条目检查缩略图缓存
    → 缓存命中：直接加载 .thumb.webp
    → 缓存未命中：后台调用 Rust thumbnail::generate() → FFmpeg/image crate → 写入缓存
  → 卡片网格渲染，hover 时 video 标签自动播放
```

### 3.2 退出恢复数据流

```
应用启动
  → Rust: backup_original_wallpaper()
    → RegQuery HKCU\Control Panel\Desktop\Wallpaper
    → 复制壁纸文件到 %AppData%/LingScape/backup/
    → 写入 backup_meta.json（路径 + 显示方式）
    → 标记 snapshot_exists = true

应用退出（托盘退出 / 关闭按钮）
  → Rust: restore_original_wallpaper()
    → 检查 snapshot_exists
    → 读取 backup_meta.json
    → SystemParametersInfoW(SPI_SETDESKWALLPAPER) 恢复
    → 删除备份文件
    → 标记 snapshot_exists = false

应用崩溃后重启
  → Rust: check_crash_recovery()
    → 检测 snapshot_exists = true
    → 弹出对话框询问是否恢复
```

### 3.3 桌面整理数据流

```
用户进入整理模式
  → DesktopOrganizerPage.vue 加载 partitions[]
  → Rust: enumerate_desktop_icons() → 获取所有图标名称和屏幕坐标
  → Rust: create_partition_window() → 为每个分区创建透明 Win32 窗口
  → 图标拖入分区 → 更新 partition_icons.json
  → 分区卷起/展开 → SetWindowPos 调整窗口高度
  → Explorer 重启 → 看门狗检测 → 重建分区窗口

分区布局持久化
  → partition_layout.json 存储：分区 ID、名称、颜色、透明度、位置、大小、卷起状态
  → partition_icons.json 存储：图标路径 → 分区 ID 映射
```

### 3.4 定时轮换数据流

```
用户配置轮换
  → SettingsPage.vue → invoke('set_rotate_config', {...})
  → Rust: auto_rotate.rs 启动定时器
  → 定时器触发 → 从轮换来源选择下一张壁纸
  → invoke('set_wallpaper', { id }) → 复用现有引擎
```

## 4. 模块与文件布局

### 4.1 模块 slug 映射

| PRD 中文模块 | 代码目录 (slug) | V2 变更 |
|-------------|----------------|---------|
| 壁纸库管理 | `src/views/LibraryPage.vue` + `src/stores/wallpaper.ts` | 升级为卡片网格 + 详情页 |
| 桌面整理 | `src-tauri/src/desktop_organizer.rs` + `src/views/DesktopOrganizerPage.vue` | **新增** |
| 基础设置 | `src/views/SettingsPage.vue` + `src/stores/settings.ts` | 新增退出恢复/轮换/多显示器/快捷键/更新检测 |
| API Key管理 | `src/views/ApiKeysPage.vue` + `src/stores/api-keys.ts` | 新增 DeepSeek/可灵卡片 |
| AI生成管线 | `src-tauri/src/api/` + `src/views/AiCreatePage.vue` | 新增进度反馈/重新生成 |
| 壁纸播放引擎 | `src-tauri/src/wallpaper_engine.rs` + `video_player.rs` | 新增播放调节 IPC |

### 4.2 V2 新增/修改文件

| 路径 | 职责 | Spec |
|------|------|------|
| `src-tauri/src/thumbnail.rs` | 缩略图生成（FFmpeg + image crate） | WL-002 |
| `src-tauri/src/wallpaper_backup.rs` | 原始壁纸快照备份与恢复 | SET-006 |
| `src-tauri/src/desktop_organizer.rs` | 分区窗口创建/管理/图标枚举 | DO-001~004 |
| `src-tauri/src/auto_rotate.rs` | 定时轮换调度 | SET-007 |
| `src-tauri/src/update_checker.rs` | 版本检测（GitHub Release API） | SET-009 |
| `src-tauri/src/api/deepseek.rs` | DeepSeek API 调用 | API-003 |
| `src-tauri/src/api/kling.rs` | 可灵视频生成 API | API-003 |
| `src/components/wallpaper/WallpaperCard.vue` | **修改**：缩略图、hover 预览、角标、收藏按钮 | WL-002/004 |
| `src/views/WallpaperDetailPage.vue` | 壁纸详情页 | WL-003 |
| `src/views/DesktopOrganizerPage.vue` | 桌面整理主页面 | DO-001 |
| `src/components/desktop/PartitionContainer.vue` | 分区容器组件 | DO-001 |
| `src/components/desktop/FolderPortal.vue` | 文件夹门户组件 | DO-002 |
| `src/components/desktop/RuleEditor.vue` | 自动规则编辑器 | DO-004 |
| `src/components/settings/RotateSettings.vue` | 轮换设置面板 | SET-007 |
| `src/components/settings/MonitorConfig.vue` | 多显示器配置 | SET-008 |
| `src/components/settings/ShortcutConfig.vue` | 快捷键配置 | SET-010 |
| `src/components/ai/GenerationProgress.vue` | **修改**：分步进度条 | AI-005 |
| `src/stores/wallpaper.ts` | **修改**：收藏/标签/搜索字段 | WL-004 |
| `src/stores/settings.ts` | **修改**：新增 V2 设置项 | SET-006~010 |
| `src/stores/desktop-organizer.ts` | 桌面整理状态管理 | DO-001~004 |
| `src/i18n/locales/zh-CN.json` | **修改**：新增 V2 文案 | 全局 |

### 4.3 测试策略

| 用途 | 框架 | 配置来源 |
|------|------|----------|
| 实现 + 验收 | vitest | `specs/.wetspec.yaml` → `unit_test`（待配置） |

build：`pnpm test:unit`；verify：`wetspec verify <spec.yaml> --root .`

## 5. 配置与常量

| 名称 | 值 | 来源 |
|------|-----|------|
| `THUMBNAIL_CACHE_DIR` | `{app_data_dir}/thumbnails/` | WL-002 |
| `THUMBNAIL_FORMAT` | `webp` | WL-002 |
| `THUMBNAIL_VIDEO_SEEK_SEC` | `0.5` | WL-002 AC-004 |
| `BACKUP_DIR` | `{app_data_dir}/backup/` | SET-006 AC-001 |
| `PARTITION_LAYOUT_FILE` | `{app_data_dir}/partition_layout.json` | DO-001 |
| `PARTITION_ICONS_FILE` | `{app_data_dir}/partition_icons.json` | DO-001 |
| `DEFAULT_PEEK_HOTKEY` | `Win+Shift+P` | DO-003 AC-001 |
| `PEEK_AUTO_HIDE_MS` | `3000` | DO-003 AC-002 |
| `ROTATE_INTERVALS` | `[15min, 30min, 1h, daily, unlock]` | SET-007 AC-002 |
| `UPDATE_CHECK_URL` | GitHub Release API / 自建 JSON | SET-009 AC-002 |
| `GLOBAL_HOTKEY_SWITCH` | `Ctrl+Shift+W` | SET-010 AC-001 |
| `GLOBAL_HOTKEY_PAUSE` | `Ctrl+Shift+P` | SET-010 AC-002 |
| `GLOBAL_HOTKEY_HIDE_ICONS` | `Ctrl+Shift+H` | SET-010 AC-003 |
| `PLAYBACK_SPEED_OPTIONS` | `[0.5, 1.0, 1.5, 2.0]` | WP-004 AC-001 |
| `ADJUST_RANGE` | `-50% ~ +50%` | WP-004 AC-002 |

## 6. API / 接口

### 6.1 新增 Tauri 命令

| 命令 | 参数 | 返回 | 用途 |
|------|------|------|------|
| `generate_thumbnail` | `{ path, mediaType }` | `{ thumbPath }` | 生成缩略图 |
| `backup_original_wallpaper` | — | `{ success }` | 备份原始壁纸 |
| `restore_original_wallpaper` | — | `{ success }` | 恢复原始壁纸 |
| `check_crash_recovery` | — | `{ needsRecovery, backupPath }` | 崩溃恢复检测 |
| `create_partition` | `{ name, x, y, w, h, color, opacity }` | `{ partitionId }` | 创建分区 |
| `delete_partition` | `{ partitionId }` | `{ success }` | 删除分区 |
| `update_partition` | `{ partitionId, ...fields }` | `{ success }` | 更新分区属性 |
| `move_icon_to_partition` | `{ iconPath, partitionId }` | `{ success }` | 图标归入分区 |
| `enumerate_desktop_icons` | — | `[{ name, path, x, y }]` | 枚举桌面图标 |
| `set_rotate_config` | `{ source, interval, order, timeRange }` | `{ success }` | 配置轮换 |
| `get_rotate_config` | — | `RotateConfig` | 获取轮换配置 |
| `set_monitor_wallpaper` | `{ monitorIndex, wallpaperId }` | `SetWallpaperResult` | 单屏设壁纸 |
| `check_update` | — | `{ hasUpdate, version, url }` | 检查更新 |
| `register_global_shortcuts` | `{ shortcuts }` | `{ success }` | 注册快捷键 |
| `set_playback_params` | `{ speed, brightness, saturation, contrast }` | `{ success }` | 播放参数调节 |

### 6.2 新增事件

| 事件 | 方向 | Payload | 用途 |
|------|------|---------|------|
| `wallpaper-rotated` | Rust → Vue | `{ id, path }` | 轮换触发通知 |
| `partition-layout-changed` | Vue → Rust | `{ partitions }` | 分区布局变更 |
| `explorer-restarted` | Rust → Vue | `{}` | Explorer 重启通知 |
| `update-available` | Rust → Vue | `{ version, url }` | 新版本可用 |
| `shortcut-triggered` | Rust → Vue | `{ action }` | 快捷键触发 |

## 7. 验收映射

### WL-002 壁纸库视觉化升级

| AC ID | 设计要点 | 测试嵌套 |
|-------|----------|----------|
| AC-001 | 卡片网格默认展示 | `describe('WL-002')` → `describe('AC-001: 卡片网格默认展示')` |
| AC-002 | 网格/列表视图切换 | `describe('WL-002')` → `describe('AC-002: 视图切换')` |
| AC-003 | 卡片尺寸切换 | `describe('WL-002')` → `describe('AC-003: 卡片尺寸切换')` |
| AC-004 | MP4/WebM 缩略图生成 | `describe('WL-002')` → `describe('AC-004: 视频缩略图生成')` |
| AC-005 | GIF 缩略图生成 | `describe('WL-002')` → `describe('AC-005: GIF 缩略图生成')` |
| AC-006 | 类型角标 | `describe('WL-002')` → `describe('AC-006: 类型角标')` |
| AC-007 | hover 动态预览 | `describe('WL-002')` → `describe('AC-007: hover 预览')` |
| AC-009 | 默认排序 | `describe('WL-002')` → `describe('AC-009: 默认排序')` |

### SET-006 退出恢复原始桌面

| AC ID | 设计要点 | 测试嵌套 |
|-------|----------|----------|
| AC-001 | 启动时备份 | `describe('SET-006')` → `describe('AC-001: 启动备份')` |
| AC-002 | 正常退出恢复 | `describe('SET-006')` → `describe('AC-002: 退出恢复')` |
| AC-005 | 崩溃恢复检测 | `describe('SET-006')` → `describe('AC-005: 崩溃恢复')` |

### DO-001 桌面分区整理

| AC ID | 设计要点 | 测试嵌套 |
|-------|----------|----------|
| AC-003 | 拖拽创建分区 | `describe('DO-001')` → `describe('AC-003: 创建分区')` |
| AC-006 | 卷起/展开 | `describe('DO-001')` → `describe('AC-006: 卷起展开')` |
| AC-008 | 图标拖入分区 | `describe('DO-001')` → `describe('AC-008: 图标拖入')` |
| AC-011 | 删除分区不删文件 | `describe('DO-001')` → `describe('AC-011: 删除分区')` |
| AC-013 | Explorer 重启恢复 | `describe('DO-001')` → `describe('AC-013: Explorer 恢复')` |

### API-003 多平台API管理

| AC ID | 设计要点 | 测试嵌套 |
|-------|----------|----------|
| AC-001 | DeepSeek 平台卡片 | `describe('API-003')` → `describe('AC-001: DeepSeek 卡片')` |
| AC-002 | 可灵平台卡片 | `describe('API-003')` → `describe('AC-002: 可灵卡片')` |
| AC-003 | 生成模式下拉 | `describe('API-003')` → `describe('AC-003: 生成模式')` |

## 8. 风险与回滚

| 风险 | 影响 | 缓解措施 |
|------|------|---------|
| FFmpeg 包体增加约 40MB | 安装包变大 | 可选：Windows Media Foundation 替代方案（Q-V2-02） |
| 桌面整理视觉覆盖式在 Win11 24H2+ 兼容性 | 分区显示异常 | 先 Win10/Win11 21H2 测试，逐步适配新版本 |
| 多显示器动态壁纸双屏同时播放性能 | CPU/GPU 占用翻倍 | 限制副屏帧率或分辨率；提供「副屏仅静态」选项 |
| 可灵视频生成成本高（$0.049/秒） | 用户误操作产生高额费用 | UI 醒目展示成本预估（Q-V2-05） |
| Explorer 重启后分区恢复延迟 | 用户感知到分区消失 | 看门狗 3s 检测 + 快速重建 |
| 全局快捷键与其他应用冲突 | 快捷键无效 | 设置页支持自定义修改（AC-005） |

