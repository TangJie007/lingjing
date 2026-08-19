# local-library-delete

阶段三收尾：阶段二已实现 `library::remove_item` 但未暴露前端调用；阶段三把删除按钮加到 `LocalLibraryView` 并暴露成 command `remove_library_item`。

## 范围

- `src-tauri/src/lib.rs` 注册 `remove_library_item`
- `src-tauri/permissions/engine.toml` 加 `"remove_library_item"`
- `src/components/LocalLibraryView.vue` hover 出现删除按钮
- `src/composables/useEngine.ts` 暴露 `removeLibraryItem(id)`

## 条款

- MUST：删除按钮（🗑）hover/focus 时显示在卡片右上角
- MUST：点击调 `window.confirm` 二次确认
- MUST：确认后调 `remove_library_item`；成功则前端 `localItems` 过滤该 id
- MUST：若被删项是 current，自动切到 localItems[0] 或 catalog[0]
- MUST：物理文件同步删除

## 验收

| ID | 条款 | 验证方式 |
|----|------|----------|
| DEL-1 | remove_library_item 命令存在 | 静态读 |
| DEL-2 | 删除按钮 hover 出现 | 手动 |
| DEL-3 | 确认后卡片消失 | 手动 |
| DEL-4 | 物理文件被删除 | 手动 |
| DEL-5 | 当前壁纸被删时切到下一项 | 手动 |
