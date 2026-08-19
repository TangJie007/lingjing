# shell-rebrand

把应用名"灵境 LingScape"统一为"灵镜 LINGJING"。仅 UI 层面改动。

## 范围

- `src-tauri/tauri.conf.json` 的 `windows[0].title` 改为 "灵镜 LINGJING"
- `src/components/TopBar.vue` 的 Logo 行改为 `<span class="text-sm font-semibold tracking-wide text-[var(--text)]">灵镜 LINGJING</span>` + 副文 `<span class="text-[11px] text-[var(--text-2)]">动态壁纸</span>`
- 所有 Tauri 窗口初始 title 用该文案

## 条款

- MUST：Tauri 窗口标题与顶栏 Logo 文字均为"灵镜 LINGJING"
- MUST：项目内不再出现"灵境" / "LingScape" 字样（grep `灵境` 与 `LingScape` 0 结果）
- MUST NOT：Tauri bundle identifier `com.lingscape.app`、npm 包名 `ling-scape` 保持不变（仅 UI 改名，不破坏已发布安装）

## 验收

| ID | 条款 | 验证方式 |
|----|------|----------|
| BR-1 | Tauri windows[0].title === "灵镜 LINGJING" | grep tauri.conf.json |
| BR-2 | TopBar.vue 第 1 行 .text-sm === "灵镜 LINGJING" | 静态读 |
| BR-3 | 项目根目录 grep "灵境" 0 命中 | rg "灵境" |
| BR-4 | 项目根目录 grep "LingScape" 0 命中 | rg "LingScape" |
| BR-5 | tauri.conf.json bundle identifier 仍是 com.lingscape.app | grep |
| BR-6 | package.json name 仍是 ling-scape | grep |
