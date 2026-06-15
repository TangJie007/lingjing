# 实现任务：灵境（LingScape）V2.0

**设计文档**：`design.md`
**Spec**：`specs/` 下全部 8 模块 36 功能（V2 新增 15 + 修改 1）
**技术栈**：Tauri (Rust) + Vue 3 + Vite

---

## 任务清单

### 阶段一：壁纸库视觉化（WL-002, WL-003, WL-004）

- [ ] 1. **WL-002 缩略图生成（Rust）**：`src-tauri/src/thumbnail.rs` → FFmpeg 提取 MP4/WebM 第 0.5 秒帧 + image crate 提取 GIF 第一帧 → WebP 编码缓存
- [ ] 2. **WL-002 卡片网格升级（Vue）**：`WallpaperCard.vue` 改造 → 缩略图展示 + 类型角标（GIF/MP4/WebM）+ AI 标识 + 收藏按钮 + 更多菜单
- [ ] 3. **WL-002 hover 动态预览**：卡片 hover 0.5s 后 `<video>` 静音自动播放，移出停止
- [ ] 4. **WL-002 多维度排序**：导入时间降序（默认）+ 名称/大小/最近使用排序 → `LibraryPage.vue` 排序下拉
- [ ] 5. **WL-002 视图切换**：网格/列表双视图 + 卡片尺寸大/中/小三档 → localStorage 持久化偏好
- [ ] 6. **WL-003 壁纸详情页**：`WallpaperDetailPage.vue` → 左侧大图预览 + 右侧元数据面板 + 路由 `/wallpaper/:id`
- [ ] 7. **WL-003 AI 方案信息展示**：原始描述、优化 Prompt、风格标签、色调色块、AI 模型 → 复制 Prompt + 重新生成按钮
- [ ] 8. **WL-003 动态播放控制**：详情页 MP4/GIF 自动播放 + 进度条 + 暂停/播放 + 键盘快捷键（空格/左右箭头/Esc）
- [ ] 9. **WL-004 收藏与标签**：`wallpaper.ts` store 扩展 → 收藏字段 + 标签数组 + 搜索过滤逻辑
- [ ] 10. **WL-004 标签筛选栏**：`LibraryPage.vue` 顶部标签筛选 + 搜索框 → 按名称/标签/AI描述匹配
- [ ] 11. 编写 WL-002~004 AC 测试，运行 `pnpm test:unit`

### 阶段二：退出恢复 + 基础设置补充（SET-006, SET-007, SET-008, SET-009, SET-010）

- [ ] 12. **SET-006 原始壁纸快照（Rust）**：`src-tauri/src/wallpaper_backup.rs` → 启动时 RegQuery 备份壁纸路径 + 复制文件到 backup/
- [ ] 13. **SET-006 退出恢复（Rust）**：退出时 `SPI_SETDESKWALLPAPER` 恢复 + 清除快照 + 崩溃恢复检测
- [ ] 14. **SET-006 设置开关（Vue）**：`SettingsPage.vue` 新增「退出恢复」+「退出确认」开关
- [ ] 15. **SET-007 定时轮换（Rust）**：`src-tauri/src/auto_rotate.rs` → 定时器 + 轮换来源选择 + 顺序/随机
- [ ] 16. **SET-007 轮换设置（Vue）**：`RotateSettings.vue` → 来源/间隔/顺序/时间段配置
- [ ] 17. **SET-008 多显示器配置（Vue）**：`MonitorConfig.vue` → 检测显示器数量 + 每屏壁纸预览 + 独立选择 + 同步开关
- [ ] 18. **SET-008 多显示器后端**：`wallpaper_engine.rs` 扩展 → `set_monitor_wallpaper` 命令，复用 MPV 多屏架构
- [ ] 19. **SET-009 更新检测（Rust）**：`src-tauri/src/update_checker.rs` → 启动时 HTTP GET GitHub Release API
- [ ] 20. **SET-009 更新横幅（Vue）**：顶部横幅组件 → 新版本提示 + 下载链接 + 可关闭
- [ ] 21. **SET-010 快捷键注册（Rust）**：`tauri-plugin-global-shortcut` → Ctrl+Shift+W/P/H 全局注册
- [ ] 22. **SET-010 快捷键配置（Vue）**：`ShortcutConfig.vue` → 自定义快捷键绑定
- [ ] 23. 编写 SET-006~010 AC 测试，运行 `pnpm test:unit`

### 阶段三：桌面整理核心（DO-001）

- [ ] 24. **DO-001 桌面图标枚举（Rust）**：`desktop_organizer.rs` → `FindWindow(ProgMan)` → `SysListView32` 获取图标名称和坐标
- [ ] 25. **DO-001 分区窗口创建（Rust）**：`WS_POPUP + WS_EX_LAYERED + WS_EX_TOOLWINDOW` 透明窗口 → Z-order 桌面层级
- [ ] 26. **DO-001 分区拖拽创建（Vue）**：`DesktopOrganizerPage.vue` → 整理模式 + 十字光标拖拽绘制矩形 + 命名输入框
- [ ] 27. **DO-001 分区视觉渲染**：标题栏（图标+名称+菜单）+ 半透明背景 + 圆角边框 + 自定义颜色/透明度
- [ ] 28. **DO-001 卷起/展开**：双击标题栏 → 只显示标题栏（28px）/ 展开完整分区
- [ ] 29. **DO-001 图标拖入分区**：拖桌面图标入分区 → 更新 `partition_icons.json` → 视觉上图标在分区内
- [ ] 30. **DO-001 布局持久化**：`partition_layout.json` + `partition_icons.json` → 启动时自动恢复
- [ ] 31. **DO-001 Explorer 重启恢复**：看门狗检测 Shell_TrayWnd → 3s 内重建分区窗口
- [ ] 32. **DO-001 隐藏/显示图标 + 对齐排序**：一键隐藏所有图标 + 按名称/类型/大小/日期排序
- [ ] 33. 编写 DO-001 AC 测试，运行 `pnpm test:unit`

### 阶段四：桌面整理高级（DO-002, DO-003, DO-004）

- [ ] 34. **DO-002 文件夹门户（Rust）**：选择本地文件夹 → 读取目录内容 → 分区内实时展示文件列表
- [ ] 35. **DO-002 文件夹门户交互**：双击打开文件 + 拖入移动文件 + 外部变化实时刷新（文件系统 Watcher）
- [ ] 36. **DO-003 Peek 功能（Rust）**：注册 Win+Shift+P 热键 → 所有分区临时置顶 → 3s 无操作自动隐藏
- [ ] 37. **DO-003 Peek 设置（Vue）**：自定义热键 + 显示时长选择
- [ ] 38. **DO-004 自动整理规则（Rust）**：文件系统 Watcher 监听桌面 → 按规则匹配 → 自动归类
- [ ] 39. **DO-004 规则编辑器（Vue）**：`RuleEditor.vue` → 文件类型/关键词规则 + 推荐模板 + 启用开关
- [ ] 40. 编写 DO-002~004 AC 测试，运行 `pnpm test:unit`

### 阶段五：AI 体验升级 + 多平台 API（AI-004, AI-005, API-003）

- [ ] 41. **API-003 DeepSeek 接入（Rust）**：`src-tauri/src/api/deepseek.rs` → DeepSeek-V4 API 调用 + 结构化响应解析
- [ ] 42. **API-003 可灵接入（Rust）**：`src-tauri/src/api/kling.rs` → 可灵 2.6 Pro 视频生成 API
- [ ] 43. **API-003 平台卡片（Vue）**：`ApiKeysPage.vue` 新增 DeepSeek + 可灵卡片 → Key 输入/掩码/测试连接
- [ ] 44. **API-003 生成模式下拉**：`AiCreatePage.vue` 新增图片模式/视频模式切换 + 模型偏好记忆
- [ ] 45. **AI-004 重新生成**：详情页「用此方案重新生成」→ 预填方案到 AI 创作页 → 微调后生成
- [ ] 46. **AI-005 生成进度**：`GenerationProgress.vue` 改造 → 第一步状态+用时 + 第二步进度条+预计时间 + 取消按钮
- [ ] 47. 编写 API-003 + AI-004~005 AC 测试，运行 `pnpm test:unit`

### 阶段六：播放调节 + 收尾（WP-004, 集成测试）

- [ ] 48. **WP-004 播放调节（Rust）**：MPV IPC 设置 speed/brightness/saturation/contrast 属性
- [ ] 49. **WP-004 播放调节（Vue）**：`SettingsPage.vue` 新增速度/亮度/饱和度/对比度滑块 → 实时预览
- [ ] 50. **集成测试**：全功能回归 → 向导 → 壁纸库浏览 → AI 生成 → 设壁纸 → 桌面整理 → 退出恢复
- [ ] 51. 运行全部 AC 验收：`wetspec verify specs/ --root .`

---

## 完成标准

- 所有勾选任务完成
- `pnpm test:unit` 全部通过（覆盖所有 auto AC）
- `wetspec verify <spec.yaml>` 对 36 个 Spec 全部返回 pass
- Spec `metadata.status` 更新为 `implemented`
- 应用可在 Windows 10/11 上完整走通：壁纸库浏览 → 详情页 → AI 生成 → 设壁纸 → 桌面整理 → 退出恢复
