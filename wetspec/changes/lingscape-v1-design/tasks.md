# 实现任务：灵境（LingScape）V1.0

**设计文档**：`design.md`
**Spec**：`specs/` 下全部 7 模块 21 功能
**技术栈**：Tauri (Rust) + Vue 3 + Vite

---

## 任务清单

### 阶段一：基础骨架（SET-005, ST-001, SET-001, OB-001）

- [ ] 1. **项目脚手架**：`npm create tauri-app` + Vue 3 + Vite + TypeScript
- [ ] 2. **Tailwind CSS 配置**：安装 + `tailwind.config.js` + `main.css`
- [ ] 3. **Vue Router + Pinia 初始化**：Hash 路由 + 基础 store
- [ ] 4. **SET-005 国际化框架**：`src/i18n/` → vue-i18n 初始化 + `zh-CN.json` 完整翻译 + `en.json` 骨架 + `formatters.ts` + 热切换
- [ ] 5. **ST-001 系统托盘**：`src-tauri/src/tray/mod.rs` → 托盘图标 + 右键菜单（暂停/切换/打开/退出）+ IPC 事件
- [ ] 6. **SET-001 开机自启**：`src-tauri/src/autostart/mod.rs` → 注册表 Run key 读写
- [ ] 7. **OB-001 欢迎页**：`WelcomeStep.vue` → 品牌动画 + 3 秒可跳过
- [ ] 8. **基础布局组件**：`AppSidebar.vue` + `AppTitlebar.vue` + `AppStatusbar.vue`
- [ ] 9. 运行 `npm run test:unit`，确保脚手架 + i18n 测试通过

### 阶段二：壁纸引擎（WP-001, WP-003, WP-002, WL-001, SET-003, SET-004）

- [ ] 10. **WP-001 静态壁纸渲染**：`src-tauri/src/wallpaper/static.rs` → Win32 API 设桌面 + 缩放模式（填充/适应/拉伸/平铺）
- [ ] 11. **WP-003 全屏检测**：`src-tauri/src/wallpaper/fullscreen.rs` → Win32 Event Hook + 暂停/恢复事件
- [ ] 12. **WP-002 视频壁纸播放**：`src-tauri/src/wallpaper/video.rs` → ffmpeg 解码 + 逐帧渲染 + 循环播放
- [ ] 13. **WL-001 本地壁纸管理**：`WallpaperGrid.vue` + `stores/library.ts` → 浏览/应用/删除/导出
- [ ] 14. **SET-003 壁纸播放设置**：帧率下拉 + 缩放模式选择 → Pinia store + IPC
- [ ] 15. **SET-004 性能与存储设置**：全屏暂停开关 + 缓存上限 + 清除缓存按钮
- [ ] 16. 编写 WP-001~003 + WL-001 AC 测试，运行 `npm run test:unit`

### 阶段三：API 基础设施（API-001, API-002, OB-002, OB-004）

- [ ] 17. **API-001 加密存储**：`src-tauri/src/crypto/mod.rs` → ring AEAD 加解密 + IPC 命令
- [ ] 18. **API-001 前端**：`PlatformCard.vue` + `KeyInput.vue` + `stores/api-keys.ts` → 掩码显示 + 粘贴去空格
- [ ] 19. **API-002 连接检测**：`StatusIndicator.vue` + Rust 最小请求 → 🟢🔴⚫ 指示灯
- [ ] 20. **OB-002 注册引导**：`ApiGuideStep.vue` → 视频教程 + 注册链接 + 截图教程 + Key 输入 + 连接诊断
- [ ] 21. **OB-004 本地模式**：跳过向导逻辑 → 本地导入 + 顶部常驻引导条
- [ ] 22. 编写 API-001~002 + OB-002/004 AC 测试

### 阶段四：AI 核心（AI-001, AI-003, AI-002, WL-002）

- [ ] 23. **AI-001 Rust 侧**：`src-tauri/src/api/doubao.rs` → doubao-2.0-vision / lite API 调用 + 结构化响应解析
- [ ] 24. **AI-001 前端**：`PromptInput.vue` + `ReferenceUpload.vue` + `StyleSelector.vue` + `stores/ai-pipeline.ts`
- [ ] 25. **AI-003 方案预览**：`PlanCard.vue` → prompt 卡片展示 + 手动编辑 + 确认触发生成
- [ ] 26. **AI-002 Rust 侧**：`src-tauri/src/api/seedream.rs` → Seedream 4.0/5.0 + 即梦视频 API + 图片下载
- [ ] 27. **AI-002 前端**：`GenerationProgress.vue` + `ResultGrid.vue` → 进度动画 + 网格展示 + 放大预览 + 设为壁纸
- [ ] 28. **WL-002 AI 生成历史**：`HistoryList.vue` → prompt/模型/时间/费用记录
- [ ] 29. 编写 AI-001~003 + WL-002 AC 测试，运行 `npm run test:unit`

### 阶段五：收尾（OB-003, API-003, SET-002）

- [ ] 30. **OB-003 快速上手**：`QuickStartStep.vue` → 3 个预设描述 + 一键生成第一张壁纸
- [ ] 31. **API-003 多平台卡片**：火山引擎可用 + DeepSeek/智谱/可灵灰色占位
- [ ] 32. **SET-002 API 默认值**：默认模型/生成数量/分辨率下拉 → Pinia store 持久化
- [ ] 33. **端到端测试**：向导 → 配置 Key → AI 生成 → 设桌面 全流程验证
- [ ] 34. 运行全部 AC 验收：`wetspec verify specs/ --root .`

---

## 完成标准

- 所有勾选任务完成
- `npm run test:unit` 全部通过（覆盖所有 auto AC）
- `wetspec verify <spec.yaml>` 对 21 个 Spec 全部返回 pass
- Spec `metadata.status` 更新为 `implemented`
- 应用可在 Windows 10/11 上完整走通：安装 → 向导 → 配置 API Key → AI 生成 → 设壁纸 → 托盘管理
