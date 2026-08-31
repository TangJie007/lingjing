# Win11 Explorer 原生右键菜单实现记录

## 背景

桌面整理的 Fence 窗口不是普通顶层应用窗口。启用桌面整理后，Fence 会被挂到 Explorer 桌面层级里：

```text
SHELLDLL_DefView
├─ SysListView32        # Explorer 原生桌面图标列表
└─ desktop-fence        # LingScape Fence/Tauri 窗口
```

旧实现通过 Shell COM 的 `IContextMenu + TrackPopupMenuEx` 在我们自己的 helper 进程里弹菜单。这个方式能拿到系统 Shell 命令，但 UI 是经典 Win32 菜单，不是 Windows 11 Explorer 的最新菜单。

如果想打开 Windows 11 最新右键菜单，关键是不要由 LingScape 自己绘制菜单，而是让 Explorer 的桌面窗口自己处理右键。

## 当前方案

图标右键时，前端先调用：

```text
show_desktop_explorer_context_menu
```

如果 Explorer 路径失败，再回退到旧的：

```text
show_desktop_native_context_menu
```

空白区域右键仍保留现有自绘菜单逻辑。

前端入口在：

```text
src/fences/FencesApp.vue
src/fences/useShellContextMenu.ts
```

后端入口在：

```text
src-tauri/src/desktop_organize/menu_commands.rs
src-tauri/src/shell_menu/context.rs
```

权限配置在：

```text
src-tauri/permissions/engine.toml
```

## 核心流程

1. 用户在 Fence 图标上右键。
2. 前端从 DOM `.cell` 上取得真实文件 `path`。
3. 后端通过 `IShellWindows` 找到 Explorer 的桌面窗口。
4. 继续取得：

```text
IShellWindows
-> IServiceProvider
-> IShellBrowser
-> IShellView
-> IFolderView
```

5. 遍历 Explorer 桌面 `IFolderView` 中的 item。
6. 读取每个 item 的 `SHGDN_FORPARSING` 路径。
7. 和 Fence 传入的真实路径做规范化匹配。
8. 匹配成功后调用 `IFolderView::SelectItem`，让 Explorer 真的选中该桌面文件。
9. 向 Explorer 的 `SysListView32` / `SHELLDLL_DefView` 发送 `WM_CONTEXTMENU`。
10. 由 Explorer 自己弹出右键菜单。

因为菜单由 Explorer 自己生成，所以在 Windows 11 上才有机会显示最新 UI，并且“共享”等命令能拿到 Explorer 的真实选中项。

## 为什么之前共享没反应

之前只是把 `WM_CONTEXTMENU` 转发给 Explorer，但没有把 Fence 图标对应的真实路径同步成 Explorer 的选中项。

结果是 Explorer 只知道“桌面某个坐标被右键了”，不知道用户点的是哪个文件。桌面图标又处于隐藏状态，Fence 图标位置也不等于 Explorer 原生 `SysListView32` 里的 item 位置，所以共享命令拿不到有效文件数据。

修复点就是先用 `IFolderView::SelectItem` 选中真实 item，再让 Explorer 弹菜单。

## 关键日志

匹配成功时应看到：

```text
[shell-menu] Explorer desktop item selected path=...
```

没有找到对应 Explorer item 时会看到：

```text
[shell-menu] Explorer desktop item not found path=...
```

选中流程异常时会看到：

```text
[shell-menu] Explorer desktop item select failed: ...
```

如果新命令没有加入 capability，前端会提示：

```text
操作未被允许，检查权限
```

对应修复是在 `src-tauri/permissions/engine.toml` 的 `commands.allow` 中加入：

```text
show_desktop_explorer_context_menu
```

## 限制和注意事项

- 这条路径依赖 Explorer 的内部桌面视图行为，属于系统集成方案，不是官方“弹出 Win11 新菜单”的独立 API。
- 如果 Explorer 没有返回对应 item，菜单可能退化成桌面空白菜单或旧菜单回退。
- Fence 图标必须对应真实桌面路径；虚拟图标、特殊 Shell 项、路径格式不同的项可能需要额外映射。
- 原有 `show_desktop_native_context_menu` 仍保留，作为失败回退和兼容路径。
- 旧的 Shell COM 菜单能执行很多命令，但不保证显示 Win11 最新 UI。

## 验证方式

1. 启动 `npm run tauri dev`。
2. 开启桌面整理。
3. 右键 Fence 中的真实桌面文件。
4. 检查是否出现 Explorer 风格的 Windows 11 菜单。
5. 点击“共享”，确认分享面板能拿到该文件。
6. 查看终端日志是否出现 `Explorer desktop item selected`。

