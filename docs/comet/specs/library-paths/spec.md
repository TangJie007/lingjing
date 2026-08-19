# library-paths

阶段三：设置页「更改壁纸路径」 + 导入策略切换（复制 / 仅引用）+ 迁移模态。顺手收尾阶段二「本地文件被复制到 app_data 但原文件仍留」的遗留。

## 范围

- `src-tauri/src/paths.rs` 新文件（set_library_dir / migrate_library / MigrationPlan / MigrationReport）
- `src-tauri/src/settings.rs` 加 `library_dir_override` 字段
- `src-tauri/src/library.rs` import 切到「按 settings.importCopyToData 决定复制或引用」
- `src/components/SettingsView.vue` 路径 input + 更改按钮 + 复制/引用 toggle
- `src/components/MigrationModal.vue` 新组件

## 条款

### 路径与持久化

- MUST：`settings.libraryDirOverride` 默认 `null`，回退到 `app_data_dir/library`
- MUST：`get_app_paths` 返回当前 `libraryDir`（受 override 影响）
- MUST：`getAppPaths` 命令从 `settings::library_root` 解析

### 迁移

- MUST：`set_library_dir(new_dir)` 创建目标目录 → 写 settings → 返回 `MigrationPlan { from_dir, to_dir, files[] }`
- MUST：`migrate_library(keep_originals)` 遍历 plan.files 复制；`keep_originals=true` 跳过已存在，`false` 覆盖；完成后 `emit("library-migration-progress", { done, total, relPath })`
- MUST：进度事件 `done === total` 时结束；前端模态根据 `done/total` 渲染进度条
- MUST：返回值 `MigrationReport { copied, skipped, failed, errors }`

### 导入策略

- MUST：`importCopyToData = true` → 仍复制到 `library_root`（兼容现状）
- MUST：`importCopyToData = false` → 不复制，仅 `media_src` / `path` 记录原始路径
- MUST：策略改动后下一次导入即生效，不影响已有项

### UI

- MUST：「更改路径」按钮打开目录选择器 → `set_library_dir` → 弹 `MigrationModal`
- MUST：模态 3 个按钮：取消 / 迁移并保留 / 迁移（删除原文件）
- MUST：迁移失败不影响设置已写入；前端根据 `report.failed > 0` 弹错误 toast

## 验收

| ID | 条款 | 验证方式 |
|----|------|----------|
| PATH-1 | set_library_dir 写 settings | 静态读 |
| PATH-2 | migrate_library 复制 | 手动 |
| PATH-3 | 模态进度事件 | 手动 |
| PATH-4 | importCopyToData=false 时原文件不动 | 手动 |
| PATH-5 | 路径更改后 get_app_paths 返回新路径 | 手动 |
| PATH-6 | 迁移后旧文件可被新进程读取 | 手动 |
