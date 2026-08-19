# design-tokens

把 `src/styles/global.css` 升级为与设计稿 1:1 的 Token 体系。覆盖浅色 / 深色双套，覆盖颜色 / 字体 / 间距 / 圆角 / 阴影 / 动效 / 字号 / 状态色。

## 范围

- `src/styles/global.css` 全面重写
- Tailwind v4 `@theme` 注册与原生 CSS 变量统一
- `--bg / --surface / --surface-2 / --border / --border-strong / --primary / --primary-hover / --primary-soft / --secondary / --text / --text-2 / --text-3 / --success / --warning / --error / --info` 全套
- `--dur-fast 150ms / --dur-base 280ms / --dur-slow 520ms / --ease / --ease-out / --ease-spring`
- `--s1..s12` 间距、 `--r-sm/md/lg/xl/pill` 圆角、 `--sh-sm/md/win` 阴影
- `--font` Inter / Segoe UI / Microsoft YaHei / PingFang SC / system-ui
- 状态色：success #16A34A / warning #D97706 / error #DC2626 / info #0891B2（light）；深色对应
- `--text-3`（设计稿新引入，#9CA3AF / 深色 #6B7280）

## 条款

- MUST：`global.css` 顶部 :root 含设计稿 1:1 全部 token
- MUST：深色 `[data-theme="dark"]` 覆盖所有同名 token
- MUST：所有组件颜色/间距/圆角/阴影走 `var(--xxx)`，不得硬编码 hex（grep 验证）
- MUST：动效铁律 `--dur-*` / `--ease*` 全应用，无组件自定义 duration 数字

## 验收

| ID | 条款 | 验证方式 |
|----|------|----------|
| T-1 | :root 包含 --bg / --surface / --surface-2 / --border / --border-strong | grep |
| T-2 | :root 包含 --primary / --primary-hover / --primary-soft / --secondary | grep |
| T-3 | :root 包含 --text / --text-2 / --text-3 | grep |
| T-4 | :root 包含 --success / --warning / --error / --info | grep |
| T-5 | :root 包含 --dur-fast 150ms / --dur-base 280ms / --dur-slow 520ms | grep |
| T-6 | :root 包含 --ease / --ease-out / --ease-spring | grep |
| T-7 | :root 包含 --s1..s12 间距变量 | grep |
| T-8 | :root 包含 --r-sm / --r-md / --r-lg / --r-xl / --r-pill | grep |
| T-9 | :root 包含 --sh-sm / --sh-md / --sh-win | grep |
| T-10 | :root 包含 --font 含 Inter/YaHei/PingFang | grep |
| T-11 | [data-theme="dark"] 覆盖 --bg / --surface / --primary / --text | grep |
| T-12 | [data-theme="dark"] --primary 是 #818CF8（深色用亮色变体） | grep |
| T-13 | 全项目 grep `#[0-9A-Fa-f]{6}` 0 命中（颜色全部 var） | rg |
| T-14 | 全项目 grep `transition.*[0-9]+ms` 0 命中（duration 全部 var） | rg |
