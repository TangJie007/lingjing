# 桌面整理：图标映射原理与方案对照

> 本文档说明 Windows 桌面图标如何映射到真实软件、腾讯桌面整理的典型做法，以及 ling-scape 当前实现的对照与可借鉴项。
>
> 相关代码：`src-tauri/src/desktop_organize/`、`src/fences/`

---

## 1. 核心结论

**桌面整理工具通常不会自建「图标 → 软件」映射库。**

无论腾讯桌面整理还是 ling-scape，本质都是：

1. 扫描桌面目录中的真实条目（`.lnk`、`.exe`、文件夹、文档等）
2. 通过 Windows Shell API 获取显示名与图标
3. 在 UI 层（格子 / Fence）中重新分类展示
4. 用户点击时，将**桌面项路径**交还给 Shell 打开；由系统解析 `.lnk` 目标并启动程序

因此：**映射对象是桌面上的路径，不是解析后的 exe 路径。**

### 1.1 产品边界（ling-scape）

ling-scape **桌面整理模块完全离线运行**，不依赖网络：

- 扫描、分类、打开、拖放、右键菜单均基于本地 Shell
- **不需要**腾讯的「移入在线」「一键上云」「云文件同步」等能力
- 离线环境下桌面整理功能应 **100% 可用**

> **与「在线壁纸」无关：** 主应用另有可选模块「在线壁纸」（`settings.onlineEnabled`，默认 `false`），属于壁纸浏览/点赞，**不参与**桌面图标映射，也不应混入本文档的桌面整理方案。

---

## 2. Windows 桌面图标是什么

| 类型 | 本质 | 如何对应到真实软件 |
|------|------|-------------------|
| 快捷方式 `.lnk` | 指向 exe 的小文件 | Shell 解析 `.lnk` 内的目标路径、参数、工作目录 |
| 桌面 `.exe` | 程序本体 | 路径即目标 |
| UWP / 商店应用 | 特殊快捷方式 | 通过 AppUserModelID 启动 |
| 文件夹 | 真实目录 | 资源管理器打开该路径 |
| 系统图标（此电脑/回收站） | Shell 命名空间 | `::{CLSID}` 或 `shell:{CLSID}` |

`.lnk` 遵循 Microsoft [MS-SHLLINK](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-shllink/) 规范，关键字段包括：

- **目标路径**（如 `C:\Program Files\WeChat\WeChat.exe`）
- **命令行参数**
- **工作目录**
- **图标位置**（可能指向某个 `.exe` / `.dll` 内的图标资源）

图标负责「显示」，真实软件由 `.lnk` 内的目标路径（或 Shell 命名空间）决定。

---

## 3. 腾讯桌面整理的典型做法

### 3.1 映射链路

```
扫描桌面目录
    ↓
识别类型 (.lnk / .exe / 文件夹 / 文档…)
    ↓
Shell 取显示名 + 图标
    ↓
归入格子（应用 / 文档 / 文件夹…）
    ↓
用户点击 → ShellExecute(桌面路径) → 系统解析 .lnk → 启动目标程序
```

### 3.2 产品特性

| 特性 | 说明 | ling-scape 是否借鉴 |
|------|------|-------------------|
| 一键整理 | 按文件类型自动分桶 | ✓ 已有类似逻辑 |
| 文件夹映射 | 格子实时展示 D/E 盘某目录内容，文件仍在原磁盘 | ✓ 待实现（P1） |
| 最近文档 | 展示近期打开/修改的文件 | ✓ 待实现（P1） |
| 系统图标 | 此电脑、回收站等与系统桌面一致 | ✓ 已用 CLSID |
| **移入在线 / 一键上云** | 文件上传到腾讯云端，跨设备访问 | **✗ 不采用**（需联网，离线无意义） |
| **云文件 AI 解析** | 云端搜索、问答 | **✗ 不采用**（需后端 + 联网） |

### 3.3 常见误解

| 误解 | 实际情况 |
|------|---------|
| 腾讯维护了软件识别库 | 主要靠 `.lnk` + Shell + 扩展名分类 |
| 整理后软件被「装进格子」 | 软件仍在安装目录，只是快捷方式被重新排版 |
| 图标是软件拷贝 | 图标是 Shell 渲染，软件由 `.lnk` 指向的 exe 决定 |

---

## 4. ling-scape 当前实现

### 4.1 数据模型

```rust
// src-tauri/src/desktop_organize/types.rs
pub struct DesktopItem {
    pub name: String,      // Shell 显示名
    pub path: String,      // 桌面项路径（身份键）
    pub is_dir: bool,
    pub kind: String,      // app / folder / document / image / media / archive / other
    pub builtin: bool,     // 系统命名空间图标
    pub icon: Option<String>, // PNG data URL（缓存）
}
```

- **`path` 是身份键**：例如 `C:\Users\...\Desktop\微信.lnk`
- **不解析 `.lnk` 目标 exe** 存入数据库
- **`kind=app`** 由扩展名判定（`lnk`、`url`、`exe`、`appref-ms` 等）

### 4.2 扫描与分类

| 步骤 | 实现位置 | 说明 |
|------|---------|------|
| 扫描目录 | `scan.rs` → `desktop_scan_dirs()` | 用户 Desktop + Public Desktop |
| 系统图标 | `win.rs` → `scan_builtin_desktop_icons()` | `::{CLSID}` 路径 |
| 扩展名分类 | `scan.rs` → `classify_kind()` | 白名单分桶 |
| 显示名/图标 | `win.rs` → `shell_name_and_icon()` | `SHGetFileInfo` + 多路图标抽取 |
| 图标缓存 | `icon_cache.rs` | 按 path + mtime 缓存 |
| 文件监视 | `lifecycle.rs` | `ReadDirectoryChangesW` 刷新 |

### 4.3 图标抽取（比典型方案更深）

`win.rs` 中的图标链路：

1. `SHGetFileInfoW`（显示名 + 系统图标索引）
2. `IShellItemImageFactory`（Shell 缩略图 / 图标）
3. `SHGFI_ICONLOCATION` → 从 PE 资源抽取最大尺寸图标
4. `AssocQueryString` 关联默认图标
5. 图片类文件额外生成 72px 预览

前端 `fenceItems.ts` 另有 path 级 icon 缓存，避免重扫闪烁。

### 4.4 打开方式

```rust
// src-tauri/src/desktop_organize/item_commands.rs
pub fn open_desktop_item(path: String) -> Result<(), String>
```

| 路径类型 | 当前实现 |
|---------|---------|
| 命名空间 `::{CLSID}` | `explorer.exe shell:{CLSID}` |
| 普通文件 / `.lnk` / 文件夹 | `cmd /C start "" <path>` |
| 属性页 | `ShellExecuteW("properties", path)` ✓ |

前端入口：`src/fences/FencesApp.vue` → `invoke("open_desktop_item", { path })`

### 4.5 桌面宿主架构

与腾讯/Fences 类工具类似，但 ling-scape **主动隐藏系统桌面图标**：

```
启用整理
  → set_icons_visible(false)          // 隐藏 SysListView32 图标
  → attach_fence_to_desktop(SetParent) // Fence 窗口挂到 WorkerW/Progman
  → scan_desktop_items()              // 自绘格子展示
  → start_desktop_watch()             // 监听桌面变化
```

相关代码：`lifecycle.rs`、`desktop.rs`、`win.rs`

### 4.6 系统特殊图标

使用 Shell 命名空间路径，**不**伪造托管 `.lnk`：

```rust
// src-tauri/src/desktop_organize/builtin_links.rs
CLSID_COMPUTER  = "::{20D04FE0-3AEA-1069-A2D8-08002B30309D}"
CLSID_RECYCLE   = "::{645FF040-5081-101B-9F08-00AA002F954E}"
CLSID_NETWORK   = "::{F02C1A0D-BE21-4350-88B0-7367FC96EF3C}"
```

旧版「此电脑.lnk」路径会在 layout 迁移时 rewrite 到 CLSID（见 `item_commands.rs` → `migrate_builtin_clsid_layout`）。

---

## 5. 方案对照表

| 维度 | 腾讯桌面整理 | ling-scape |
|------|-------------|------------|
| **映射语义** | 桌面项路径 → Shell 打开 | 相同 |
| **是否解析 lnk 目标** | 否（运行时由 Shell 解析） | 否 |
| **分类规则** | 文件类型分桶 | 扩展名白名单（`classify_kind`） |
| **扫描范围** | Desktop + Public Desktop | 相同 |
| **系统图标** | 与系统桌面一致 | `::{CLSID}` + 库存图标 |
| **图标来源** | Shell 图标 + 自研渲染 | 深度 Shell 抽取 + PNG 缓存 |
| **打开方式** | `ShellExecute` 类调用 | `cmd start`（待改进） |
| **UI 宿主** | 格子叠在桌面上 | 隐藏 ListView + 自绘 Fence |
| **文件夹映射** | ✓ 核心功能 | ✗ 尚未实现 |
| **最近文档** | ✓ | ✗ 尚未实现 |
| **Shell 右键菜单** | 系统/自研混合 | 完整 COM 上下文菜单 |
| **布局持久化** | 产品内配置 | `fence_layout.json` |

---

## 6. 打开链路对照

```
腾讯（典型）:
  格子图标 ──点击──► ShellExecute(桌面.lnk) ──► 解析目标 ──► WeChat.exe

ling-scape（当前）:
  格子图标 ──双击──► open_desktop_item(path)
                        ├─ ::{CLSID} → explorer shell:{...}
                        └─ 其它     → cmd /C start "" path
                                        └─ Windows 解析 .lnk → 目标程序
```

两者「认软件」的方式相同：依赖系统对 `.lnk` 的解析，而非自建软件库。

---

## 7. 可借鉴项（按优先级）

### P0 — 直接照搬，改动小

#### 7.1 打开方式改为 ShellExecute

**现状：** 普通项走 `cmd /C start "" path`

**建议：** 统一 `ShellExecuteW(NULL, L"open", path, ...)`（`show_properties` 已有范例）

**收益：**

- 与腾讯 / 资源管理器行为一致
- UAC、工作目录、`.lnk` / `appref-ms` 解析更可靠
- 少一层 `cmd` 进程

**改动文件：** `item_commands.rs`

#### 7.2 保持「路径即身份」模型

继续以 `DesktopItem.path` 为唯一身份键，**不要**额外维护 `targetExe` / 软件名映射表。

#### 7.3 保持双桌面扫描 + 扩展名分类

`scan.rs` 中 `desktop_scan_dirs()` + `classify_kind()` 已与腾讯思路一致，无需重构。

---

### P1 — 产品能力，可复用现有模块

#### 7.4 文件夹映射格子

腾讯核心功能：格子绑定任意磁盘路径，文件实体不移动。

| 已有能力 | 映射格子用法 |
|---------|-------------|
| `scan_dir()` | 扫描 `D:\Downloads` 等任意路径 |
| `DesktopItem` | `path` 指向真实文件 |
| `open_desktop_item` | 双击 Shell 打开 |
| `FenceLayout` | 新增 `mappedFolders` 配置 |
| 文件监视 | 对映射目录 `ReadDirectoryChangesW` |
| 解散格子 | 只删 layout，不删磁盘文件 |

**改动文件：** `layout.rs`、`scan.rs`、`FencesApp.vue`、`fenceLayout.ts`

#### 7.5 最近文档格子

**做法：** 读取 `%APPDATA%\Microsoft\Windows\Recent\*.lnk`，复用 `shell_name_and_icon` 取显示名与图标，打开仍走 `ShellExecute`。

**改动文件：** 新增 `recent.rs` 或扩展 `scan.rs`

---

### P2 — 体验增强

#### 7.6 一键整理 / 按时间排序

- 一键整理：清空自定义 order，回到 `classify_kind` 默认分桶
- 按修改时间分组：一周内 / 更早（只影响排序）

**改动文件：** `fenceLayout.ts`、`helpers.ts`

#### 7.7 图标两阶段加载

- 首次扫描：`SHGFI_ICON` 快速出图
- 后台/async：`extract_best_icon` 补高清图

**改动文件：** `win.rs`、`icon_cache.rs`

#### 7.8 双击隐藏

`desktop.rs` 已有 `WH_MOUSE_LL` 钩子，可对齐腾讯/Fences 的「双击空白隐藏图标」。

---

## 8. 不建议采用的方案

| 腾讯 / 常见做法 | 不建议原因 |
|----------------|-----------|
| **移入在线 / 一键上云 / 云同步** | 需联网 + 账号 + 后端；离线场景无价值，与桌面整理核心无关 |
| **云文件 AI 搜索** | 依赖云端索引，离线不可用 |
| 不隐藏系统图标，只在上面叠格子 | ling-scape 已 `SetParent` + 隐藏 ListView，改回去会双份图标 |
| 继续用 `cmd start` 打开 | 应升级为 `ShellExecute`，不是降级 |
| 伪造「此电脑.lnk」放 AppData | 已 rollback 到 CLSID，更稳定 |
| 自建「软件名 → exe」映射表 | 维护成本高，与 `.lnk` 重复 |
| 解析 `.lnk` 目标再归类 | 快捷方式改名/改目标会导致分类混乱 |

---

## 9. ling-scape 相对优势（保留）

| 能力 | 说明 |
|------|------|
| CLSID 系统图标 | 比伪造 `.lnk` 更贴近 Shell 语义 |
| 深度图标抽取 | PE 图标 + 关联图标 + 图片预览 |
| 图标缓存 | path + mtime，重扫不闪烁 |
| Shell COM 右键菜单 | 含图标、pin 条、子菜单 |
| OLE 拖放 | `RegisterDragDrop` + CF_HDROP 拖出 |
| 布局持久化 | Rust 侧 `fence_layout.json` + 前端迁移 |

---

## 10. 落地路线图

```
P0  open_desktop_item → ShellExecuteW           （~半天，兼容性提升）
P1  文件夹映射格子                                 （复用 scan_dir + layout）
P1  最近文档格子                                   （读 Recent/*.lnk）
P2  一键整理 / 按时间排序                          （纯 layout + 前端）
P3  图标两阶段加载                                 （性能优化，非必须）

不在路线图内：移入在线、一键上云、云同步、云 AI 搜索（离线产品不需要）
```

---

## 11. 关键代码索引

| 文件 | 职责 |
|------|------|
| `src-tauri/src/desktop_organize/scan.rs` | 桌面扫描、扩展名分类 |
| `src-tauri/src/desktop_organize/win.rs` | Shell 图标、Fence 窗口附着 |
| `src-tauri/src/desktop_organize/item_commands.rs` | 打开/删除/重命名/移动 |
| `src-tauri/src/desktop_organize/builtin_links.rs` | 系统命名空间 CLSID |
| `src-tauri/src/desktop_organize/lifecycle.rs` | 启用/禁用/监视/刷新 |
| `src-tauri/src/desktop_organize/layout.rs` | 格子布局持久化 |
| `src-tauri/src/desktop_organize/icon_cache.rs` | 图标 mtime 缓存 |
| `src-tauri/src/desktop.rs` | 隐藏/恢复系统桌面图标 |
| `src/fences/FencesApp.vue` | 格子 UI、打开入口 |
| `src/fences/fenceLayout.ts` | 前端布局读写 |
| `src/fences/fenceItems.ts` | 前端 icon 缓存与 diff |

---

## 12. 参考资料

- [MS-SHLLINK: Shell Link (.LNK) Binary File Format](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-shllink/)
- [腾讯电脑管家 - 桌面整理帮助](https://gj.qq.com/help/2071.html)
- 模块 README：`src-tauri/src/desktop_organize/README.md`

---

*最后更新：2026-09-01*
