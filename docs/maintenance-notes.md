# 维护备忘

迭代时优先遵守下列约束，避免重复踩坑。细节以源码为准；本页只记**易忘结论**。

## 进程与窗口

1. **关主窗 ≠ 退出**  
   `CloseRequested` 只 `hide`。壁纸 / Fence / 托盘仍在。用户再开快捷方式会二次启动 → 已用 `tauri-plugin-single-instance` 聚焦已有窗口。

2. **真退出**  
   必须走托盘「退出」或等价 cleanup：`desktop_organize::cleanup` + `wallpaper::cleanup`，再 `exit`。

3. **同 exe 多角色**  
   `main.rs` 早退：`--desktop-icons-guard`、Shell 菜单宿主。改启动参数或单实例时不要误伤这些分支。

4. **WorkerW / DefView**  
   壁纸挂 WorkerW（图标下），Fence 挂 DefView。乱改 `SetParent` / Z-order 会导致点不到图标或壁纸盖住格子。

## 打开文件 / Shell

5. **不要在主路径同步 `ShellExecute` 打开命名空间**（回收站/此电脑）  
   易与桌面 OLE / DDE 死锁（第二次打开未响应）。  
   现行：`explorer shell:RecycleBinFolder` 等 + 已开则激活；普通项 `ShellExecuteEx` + `SEE_MASK_ASYNCOK`；属性用 `rundll32 ShellExec_RunDLL properties`。

6. **`runas`（以管理员身份运行）**  
   **禁止** `SEE_MASK_FLAG_NO_UI` / 过早 `ASYNCOK`。必须允许 UAC 对话框，并等提权流程（菜单宿主内同步 `ShellExecuteEx`）。

7. **格子打开本应用**  
   `open_desktop_item` 解析 `.lnk`/exe 若指向自身 → 聚焦 `main`，不要再 spawn。

8. **前端防连点**  
   `src/fences/iconOpen.ts` 的 `beginIconOpen`；打开反馈 class：`is-pressed` / `is-selected` / `is-opening`。

## 壁纸引擎

9. **进度上报**  
   壁纸页间隔与 Rust `engine_report_progress` 均已节流；勿改回高频 `emit("engine-state")`，主 UI 会拖死。

10. **单曲循环**  
    交给 `wallpaper.html` 的 `<video loop>`；不要在 `App.vue` 用 near-end 再 `set_wallpaper`（会刷 toast / 重载）。

11. **休眠恢复**  
    `resumeIfNeeded` 只 `play()`，不要整段 `applyMedia("set")` 重载源。

12. **双屏**  
    两路 1080p `<video>` 成本高；长跑卡顿优先查内存/GPU，再考虑副屏降级。

## 桌面整理

13. **系统图标**  
    用 `::{CLSID}`，不要再造 AppData「此电脑.lnk」。说明见 `desktop-organize-reference.md`。

14. **OLE**  
    `SetParent` 后需重装 drop target（`drop_target.rs`）。拖出在 `drag.rs`。

15. **Shell 菜单宿主**  
    `::{CLSID}` 用 one-shot，避免拖死常驻 host。主进程通过 `spawn_blocking` 等宿主返回——宿主内阻塞 UI（如属性）必须快速 spawn/异步。

16. **图标恢复守卫**  
    启用整理时写 temp guard；崩溃后下次启动 `recover_icons_after_crash`。

## 前端 / 网络

17. **在线请求**  
    用 `apiFetch`（Tauri HTTP 插件），不要用浏览器 `fetch` 打跨域 API。

18. **监听引擎事件**  
    `listen` 需等 `__TAURI_INTERNALS__` 就绪（`useEngine.ts`）。

19. **模态层**  
    全局弹层 Teleport 到 `.window` 内，定位用 absolute，避免盖住整屏/托盘错位。

## 改动检查清单（建议）

- [ ] 是否引入第二实例或重复 `SetParent`？
- [ ] 打开/属性/runas 是否又变成同步堵在 UI 或宿主里？
- [ ] 壁纸进度是否重新高频 emit？
- [ ] 关闭主窗后壁纸/整理是否仍符合产品预期？
- [ ] `capabilities` / 新 command 是否已注册？
- [ ] 文档：架构级变更时更新 `architecture.md` / 本页

## 排障速查

| 现象 | 先查 |
|------|------|
| 双击桌面图标后整程序未响应 | 单实例是否生效；是否第二进程抢 WorkerW |
| 第二次点回收站卡死 | `shell_open_known_folder` / 是否退回同步 ShellExecute |
| 以管理员运行无反应 | `verbs.rs` 是否对 runas 禁了 UI |
| 主窗关了找不到 | 托盘；是否误杀进程 |
| 桌面图标消失且整理关不掉 | guard 文件、`set_desktop_organize(false)`、手动显示 ListView |
| 在线页 CORS / 失败 | 是否走了 `apiFetch`；capability `http:` |
| 托盘点了没窗 | `focus_main_window`：show + unminimize + focus |

## 相关文档

- [architecture.md](./architecture.md)
- [directory-structure.md](./directory-structure.md)
- [desktop-organize-reference.md](./desktop-organize-reference.md)
