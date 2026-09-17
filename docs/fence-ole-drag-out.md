# 分区文件拖出到资源管理器（OLE）

> 把 Fence 格子里的桌面文件拖进**已打开的资源管理器窗口**（含其他盘符文件夹）时，走 Windows OLE `DoDragDrop` + `CF_HDROP`。  
> 相关代码：`src-tauri/src/desktop_organize/drag.rs`、`drop_target.rs`，前端 `src/fences/useShellFileDrag.ts`、`FencesApp.vue`。

本文记录**现行方案**与**已踩过的坑**，避免再把「拖到 D 盘文件夹没反应」修成别的需求（例如把「文件夹」从标签拆成常驻条）。

---

## 1. 产品意图（先分清）

| 场景 | 期望 | 实现位置 |
|------|------|----------|
| 格子内排序 / 跨格子移动 | Sortable 重排 + 布局持久化 | `FenceGroup.vue` / layout |
| 拖到格子里的**文件夹单元格** | 移入该桌面文件夹 | `move_desktop_item_into_folder` |
| 拖到**外部已打开的 Explorer 窗口** | 系统复制/移动到该路径 | OLE 拖出（本文） |
| 从 Explorer 拖进桌面/格子 | 放置到桌面 | `drop_target.rs` RegisterDragDrop |

用户说「打开了 D 盘某个文件夹，把分区文件拖进去」→ 指的是第三行，**不是**格子内文件夹单元格。

---

## 2. 总体流程

格子内拖动需要同时支持 Sortable（分区内）和 Shell 拖出（分区外），因此采用 **双轨**：

```
Sortable @start
    │
    ├─ JS: useShellFileDrag.begin(path)
    │       └─ invoke arm_desktop_outgoing_drag_watch   ← 立刻武装原生监视
    │       └─ 定时 probe → is_desktop_drag_over_foreign
    │
    └─ Rust 后台线程（抗 WebView 焦点在 Explorer 时定时器被节流）
            轮询：LBUTTON 仍按下？光标在「外部窗口」？
                │
                ├─ emit desktop-outgoing-drag-handoff  （先拆 Sortable，LBUTTON 仍物理按下）
                └─ UI 线程 DoDragDrop(CF_HDROP, COPY|MOVE)
                        │
                        └─ Explorer IDropTarget 接手；本进程 drop_target 靠 OUTGOING_DRAG 拒收自吞
```

要点：

1. **OLE 必须在左键仍按下时启动**（`GetAsyncKeyState(VK_LBUTTON)`）。等用户在 Explorer 上松开再启动 → 必失败。
2. **不能只靠 WebView 的 `pointermove` / `setInterval`**：光标进入 Explorer 后，Fence WebView 常失焦，定时器被严重节流。原生 `arm_desktop_outgoing_drag_watch` 线程是主路径；JS probe 是辅路径。
3. Fence 挂在 DefView 下且近似全屏，命中测试易踩坑（见 §3）。

---

## 3. 「外部窗口」判定（最易写错）

实现：`classify_cursor_foreign` / `is_cursor_over_foreign_window`（`drag.rs`）。

### 3.1 窗口层级事实

**桌面：**

```
Progman / WorkerW
  └── SHELLDLL_DefView
        ├── SysListView32（桌面图标）
        └── 我们的 Fence（WS_CHILD）
```

**资源管理器文件夹窗口：**

```
CabinetWClass（或 ExploreWClass / Win11 XAML 壳）
  └── …
        └── SHELLDLL_DefView          ← 也有！
              └── SysListView32 / DirectUIHWND
```

### 3.2 错误写法（曾导致「拖进 D 盘没反应」）

```text
父链上遇到 SHELLDLL_DefView → 判定为桌面 → foreign = false
```

结果：光标已在 Explorer 客户区，仍认为「还在桌面」，**永不 `DoDragDrop`**。

### 3.3 正确写法

| 类名 | 含义 |
|------|------|
| `Progman` / `WorkerW` | **仅这些**算桌面根（`is_desktop_root_class`） |
| `CabinetWClass` / `ExploreWClass` / 若干 Win11 Explorer 壳类 | 外部 / Explorer（`is_explorer_frame_class`） |
| `SHELLDLL_DefView` | **不要单独当桌面**；继续向上走，看祖先是 WorkerW 还是 CabinetWClass |

额外兜底：`explorer_window_contains_point` —— `EnumWindows` 找可见的 Explorer 顶层窗，用 `GetWindowRect` + 光标点做几何命中。  
即使 `WindowFromPoint` 仍打到全屏 DefView 子窗，只要光标落在 Explorer 矩形内，也判 `foreign = true`。

### 3.4 自有窗口

`collect_own_hwnds`：Fence + wallpaper 的 HWND。命中这些 → 非外部（继续 Sortable）。

---

## 4. 武装 / 取消生命周期（第二易错）

### 4.1 命令

| 命令 | 作用 |
|------|------|
| `arm_desktop_outgoing_drag_watch` | `OUTGOING_WATCH_GEN++` 并起监视线程；**自身即可作废旧监视** |
| `cancel_desktop_outgoing_drag_watch` | 只 bump gen，停线程 |
| `try_start_desktop_file_drag_if_foreign` | JS probe 确认 foreign 后同步走 OLE |
| `is_desktop_drag_over_foreign` | 单次判定（带日志） |

### 4.2 前端纪律（`useShellFileDrag.ts`）

**正确：**

- `begin` → **直接 `armNativeWatch()`**（不要先 `cancel` 再异步 `arm`）
- `arm` 在 Rust 侧会 bump gen，自然取消上一次
- 仅在悬停格子内「文件夹单元格」时 `setSuspended(true)`（暂停 OLE，避免和格子内移入冲突）
- Sortable `@end`：**不要**再 `setSuspended(true)` 误杀监视；只 `end()`

**错误（曾导致日志只有 `cancel`、没有 `arm`）：**

```text
begin: await cancel() → then arm()     // end() 清空 activePath 后 then 直接 return，arm 永不调用
onDragEnd: setSuspended(true); end()   // 连 cancel 三次，拖出过程中监视被掐死
arm 命令里先做慢速 shell 图标预览再记日志 / 起线程  // 看起来像没 arm
```

现行 `arm`：先 bump gen + 打日志，预览只用前端 data-URL 或 1×1 PNG，**禁止**在命令线程上同步抽 Shell 大图标。

### 4.3 自拖自放

`drop_target.rs` 的 `OUTGOING_DRAG`：本进程作为拖源时，自家 `IDropTarget` 拒收，避免「拖到 Explorer 却被全屏 Fence drop target 吞掉变成空操作」。

`ole_drag::start` 用 RAII guard 置位/清除该标志。

### 4.4 路径

拖出路径去掉 `\\?\` 前缀（`strip_extended_path`）。Explorer 对扩展路径在 HDROP 里较挑剔。

---

## 5. 事件与 UI 交接

| 事件 | 时机 | 前端 |
|------|------|------|
| `desktop-outgoing-drag-handoff` | 即将 `DoDragDrop`，LBUTTON 仍按下 | 拆 Sortable / ghost（`cancelHtmlDragArtifacts`），`handoff=true`，忽略随后的 Sortable `end` 取消 |
| `desktop-outgoing-drag-done` | OLE 会话结束 | 清 session；Rust 侧会 `lifecycle::refresh` |

`DoDragDrop` 允许 `DROPEFFECT_COPY | DROPEFFECT_MOVE`，由目标（Explorer）选择；跨卷通常为复制。

---

## 6. 排障清单

拖到已打开的 Explorer **完全没反应**时，按终端日志查：

| 期望日志 | 没有则说明 |
|----------|------------|
| `arm outgoing watch gen=… path=…` | 前端没调用到 arm，或 command/权限未注册 |
| `watch gen=… thread start` | 线程没起来（arm 在校验阶段失败） |
| `watch … foreign=true` / `explorer-rect …` | 窗口判定仍错，或光标从未进入 Explorer 矩形 |
| `handoff → OLE` | 判到了 foreign 但随后 LBUTTON 已松开 / gen 被 cancel |
| `DoDragDrop finished` / `watch shell file drag finished` | OLE 跑完；若文件没到目标，查 effect / 权限 / 只读盘 |

前端控制台过滤 `[shell-drag]`：应有 `begin` → `armed watch`；拖到 Explorer 时有 `native handoff` / `probe → OLE`。

**若只有成组的 `cancel outgoing watch`、从无 `arm`：** 先查 §4.2 竞态，不要先改 UI 布局。

---

## 7. 改动约束（给未来 PR）

1. **不要**把「拖到外部文件夹」做成「把文件夹类型从标签拆成常驻条」之类的产品误解。
2. **不要**把 `SHELLDLL_DefView` 单独当作桌面判定条件。
3. **不要** `cancel().then(arm)`；**不要**在 `@end` 里无条件 `setSuspended(true)`。
4. **不要**删掉原生 watch 只留 JS probe（Explorer 上会节流）。
5. **不要**在 `arm` 同步路径做 Shell 图标/大图解码。
6. 新增拖出相关 command 时同步：`lib.rs` 注册、`permissions/engine.toml`、`capabilities/default.json`（含 `desktop-fence`）。
7. 改判定逻辑时保留 `foreign=…` 原因字符串日志，便于对照窗口链。

---

## 8. 代码索引

| 文件 | 职责 |
|------|------|
| `drag.rs` | 外部窗口判定、arm/cancel、OLE `DoDragDrop` |
| `drop_target.rs` | 拖入桌面；`OUTGOING_DRAG` 拒自吞 |
| `useShellFileDrag.ts` | begin/arm/probe/handoff |
| `FencesApp.vue` | Sortable start/end、文件夹单元格 `setSuspended` |
| `permissions/engine.toml` | `arm_desktop_outgoing_drag_watch` 等 allow |

---

## 9. 历史事故摘要（2026-09）

**现象：** 打开 D: 下文件夹，从分区拖文件进去无反应。

**根因：**

1. `SHELLDLL_DefView` 误判为桌面 → 在 Explorer 上 `foreign` 恒为 false。  
2. 前端 cancel/arm 竞态 + end 时多余 cancel → 监视经常未启动（日志只有 cancel）。

**修复方向：** 桌面根仅 `Progman`/`WorkerW` + Explorer 矩形兜底；直接 arm；end 不误杀；arm 轻量预览。
