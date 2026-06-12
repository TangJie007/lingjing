# 灵境 (LingScape) 字体与 Logo 设计方案

> **设计日期**: 2026-06-12  
> **设计师**: UI Designer  
> **版本**: V2.0 — 「灵叶」Logo 重设计

---

## 一、品牌 Logo 设计

### 1.1 设计概念：「灵叶」

Logo 的核心意象是一片**清晨沾着露水的叶子/水滴**：

- **有机形态**：不对称、流动的轮廓，提取自参考图中的水滴/叶片意象。拒绝几何冰冷，传递生命力。
- **极光渐变色谱**：从暖金 → 嫩绿 → 翡翠 → 冰青 → 雾青，模拟晨光穿透叶片时的色彩折射。
- **内部高光脉络**：一条柔和的白色曲线，像叶脉般从根部延伸到尖端，赋予呼吸感。
- **外发光滤镜**：微妙的柔和光晕，让 Logo 像晨露一样朦胧发光。

### 1.2 符号解析

| 元素 | 含义 | 视觉表达 |
|------|------|----------|
| 有机水滴/叶片 | 生命、自然、生长 | 不对称 Bezier 曲线，无直角 |
| 暖金→冰青渐变 | 晨光、创造力、AI 起点 | 5 色线性渐变，对角线方向 |
| 内部白脉 | 叶脉、呼吸、流动感 | 0.8px 描边，25% 透明度 |
| 高光覆盖层 | 晨露光泽、通透感 | 径向渐变，左上角偏白 |
| 外发光 | 灵气、朦胧、不锐利 | SVG feGaussianBlur 1.5px |

### 1.3 多尺寸适配

**主 Logo (App 启动图标 / 关于页 / Splash)**
- 尺寸: 512x512px / 1024x1024px
- 包含: 灵叶符号 + "灵境" 文字 + "LINGSCAPE" 英文

**App 图标 (48x48 / 256x256)**
- 仅保留: 灵叶符号
- 渐变保持完整，比例放大

**Titlebar / 工具栏 (22x22 / 24x24)**
- 仅保留: 灵叶符号
- 外发光 stdDeviation 降至 1px

**系统托盘 / Favicon (16x16)**
- 仅保留: 灵叶符号
- 外发光关闭，用纯色替代

**单色版本 (打印 / 水印 / 特殊场景)**
- 纯 `#34d399` (翡翠青) 填充
- 去除渐变和发光，保留轮廓

### 1.4 色彩规范

```
Logo 渐变色谱:
  #fbbf24 (暖金) → #a3e635 (嫩绿) → #34d399 (翡翠) → #22d3ee (冰青) → #67e8f9 (雾青)
  
Logo 单色填充:   #34d399 (翡翠青)
Logo 背景:       透明 或 #f5fdf8 (晨露背景)
Logo 反白:       #ffffff 用于深色背景
```

### 1.5 SVG 实现要点

```svg
<svg viewBox="0 0 48 48" xmlns="http://www.w3.org/2000/svg">
  <defs>
    <filter id="logo-glow">
      <feGaussianBlur stdDeviation="1.5" result="blur"/>
      <feMerge><feMergeNode in="blur"/><feMergeNode in="SourceGraphic"/></feMerge>
    </filter>
    <linearGradient id="logo-fill" x1="4" y1="36" x2="44" y2="8">
      <stop offset="0%" stop-color="#fbbf24"/>
      <stop offset="25%" stop-color="#a3e635"/>
      <stop offset="55%" stop-color="#34d399"/>
      <stop offset="85%" stop-color="#22d3ee"/>
      <stop offset="100%" stop-color="#67e8f9"/>
    </linearGradient>
    <radialGradient id="logo-highlight" cx="16" cy="14" r="18">
      <stop offset="0%" stop-color="white" stop-opacity="0.35"/>
      <stop offset="60%" stop-color="white" stop-opacity="0.08"/>
      <stop offset="100%" stop-color="white" stop-opacity="0"/>
    </radialGradient>
  </defs>
  <!-- 主体有机形态 -->
  <path d="M 24 3 C 14 3, 5 12, 5 24 C 5 35, 12 43, 20 45 C 28 47, 38 42, 41 33 C 44 24, 42 14, 34 8 C 30 5, 27 3, 24 3 Z"
        fill="url(#logo-fill)" filter="url(#logo-glow)"/>
  <!-- 内部脉络 -->
  <path d="M 24 6 C 28 6, 32 9, 35 15 C 38 21, 37 29, 33 35 C 30 39, 25 41, 20 40"
        fill="none" stroke="white" stroke-width="0.8" stroke-linecap="round" opacity="0.25"/>
  <!-- 高光覆盖 -->
  <path d="M 24 3 C 14 3, 5 12, 5 24 C 5 35, 12 43, 20 45 C 28 47, 38 42, 41 33 C 44 24, 42 14, 34 8 C 30 5, 27 3, 24 3 Z"
        fill="url(#logo-highlight)"/>
</svg>
```

### 1.6 与 V1「灵境之门」的对比

| 维度 | V1「灵境之门」 | V2「灵叶」 |
|------|---------------|-----------|
| 形态 | 几何矩形+三角形山峰 | 有机水滴/叶片 |
| 渐变方向 | 青→蓝→金（对角线） | 暖金→嫩绿→翡翠→冰青（对角线） |
| 视觉语言 | 窗口、画框、门户 | 自然、生命、晨露 |
| 用户感受 | 技术感、框架感 | 温度感、创造感、生长感 |
| 与亮色主题契合 | 一般（为暗色设计） | **极佳**（晨露主题的核心符号） |

### 1.5 SVG 代码 (可直接嵌入应用)

见 `lingscape-logo.svg` (已在原型中嵌入使用)

---

## 二、自定义字体方案

### 2.1 设计原则

拒绝"AI 产品标配字体"（Inter + Noto Sans），选择**有辨识度、有性格、开源可商用**的字体组合。

选型标准：
1. **辨识度**: 不是 90% 的 SaaS 产品在用的字体
2. **意境契合**: 中文字体要有"灵境"的空灵或温度感
3. **开源免费**: 可合法商用，无授权风险
4. **多字重**: 至少提供 Regular / Medium / Bold

### 2.2 字体层级

#### 层级 1: 品牌展示 (Display)
- **中文**: 得意黑 (Smiley Sans / dyh)
  - 来源: 站酷 + atelierAnchor 联合发布，SIL Open Font License
  - 特点: 几何感 + 8° 斜切，极具现代感和辨识度
  - 用途: Logo 旁英文、Splash 启动页大标题、营销物料
  - 加载: 仅加载 500/600 字重，~180KB

- **英文**: Space Grotesk
  - 来源: Google Fonts，OFL
  - 特点: 几何无衬线，字符间距略宽，有"空间感"
  - 用途: 英文品牌名、大标题、英文界面
  - 加载: 400/500/600，~90KB

#### 层级 2: 标题 (Headlines / H1-H3)
- **中文**: 霞鹜文楷 (LXGW WenKai)
  - 来源: LXGW 开源字体，SIL OFL
  - 特点: 基于 Klee One，温润优雅，有书卷气但不过分古板
  - 用途: 页面主标题、引导页文案、空状态文案
  - 加载: Regular / Bold，~4.5MB (中文字体较大，建议子集化)

- **英文**: Space Grotesk (延续)
  - 用途: 英文标题、按钮大字

#### 层级 3: 正文 (Body)
- **中文**: 霞鹜文楷 (LXGW WenKai) 或 Noto Sans SC
  - 若 WenKai 文件过大: 正文 fallback 到 Noto Sans SC
  - WenKai 用于需要"温度感"的场景（欢迎页、生成结果页）

- **英文**: Inter
  - 来源: Google Fonts，OFL
  - 特点: 屏显优化极佳，小字号清晰
  - 用途: 英文正文、描述文本、设置项
  - 加载: 400/500，~60KB

#### 层级 4: 界面标签 (UI Labels / Captions)
- **中文**: Noto Sans SC
  - 用途: 按钮文字、标签、导航、表单
  - 原因: 屏显清晰、字重齐全、文件经过 Google CDN 优化

- **英文**: Inter
  - 用途: 英文标签、工具提示

#### 层级 5: 等宽代码 (Monospace)
- **英文**: Cascadia Code
  - 来源: Microsoft，OFL
  - 特点: 连字支持、清晰的 0/O 区分、现代感
  - 用途: API Key 显示、代码块、技术参数

- **中文等宽**: 霞鹜文楷等宽 (LXGW WenKai Mono)
  - 用途: 混合中英文的等宽场景

### 2.3 字体加载策略

```html
<!-- 核心字体 (首屏必须) -->
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link href="https://fonts.googleapis.com/css2?family=Space+Grotesk:wght@400;500;600&family=Inter:wght@400;500&display=swap" rel="stylesheet">

<!-- 中文字体 (异步加载，有 fallback) -->
<link href="https://cdn.jsdelivr.net/npm/lxgw-wenkai-webfont@1.7.0/style.css" rel="stylesheet" media="print" onload="this.media='all'">

<!-- 本地字体 (Electron 应用推荐打包到本地) -->
@font-face {
  font-family: 'SmileySans';
  src: url('./fonts/SmileySans-Oblique.ttf') format('truetype');
  font-weight: 500;
  font-display: swap;
}
```

### 2.4 降级策略

```css
/* 品牌展示 */
font-family: 'SmileySans', 'Space Grotesk', 'PingFang SC', sans-serif;

/* 标题 */
font-family: 'LXGW WenKai', 'Space Grotesk', 'Noto Sans SC', sans-serif;

/* 正文 */
font-family: 'LXGW WenKai', 'Inter', 'Noto Sans SC', 'PingFang SC', sans-serif;

/* 界面标签 */
font-family: 'Inter', 'Noto Sans SC', 'PingFang SC', sans-serif;

/* 代码 */
font-family: 'Cascadia Code', 'JetBrains Mono', 'Fira Code', monospace;
```

### 2.5 性能优化

| 优化手段 | 说明 |
|----------|------|
  font-display: swap | 先用 fallback 字体渲染，加载完成后替换 |
| 子集化 | 中文字体使用 `cn-font-split` 工具按需切割，只加载用到的汉字 |
| 本地打包 | Electron 应用将字体打包到本地，避免网络依赖 |
| WOFF2 | 所有网络字体使用 WOFF2 格式，体积最小 |
| 预连接 | `preconnect` 到 fonts.googleapis.com 和 gstatic.com |

### 2.6 与竞品的字体差异化

| 产品 | 字体选择 | 问题 |
|------|----------|------|
| Wallpaper Engine | 系统默认 (Segoe UI) | 无品牌感 |
| 飞火壁纸 | 系统默认 | 无品牌感 |
| 元气壁纸 | 系统默认 | 无品牌感 |
| **灵境** | **得意黑 + 霞鹜文楷 + Space Grotesk** | **独特、有温度、有辨识度** |

---

## 三、字体在界面中的应用示例

### Splash 启动页
```
[得意黑 / Space Grotesk]
灵境
L I N G S C A P E

[霞鹜文楷]
AI 动态桌面创造者
```

### 主界面标题
```
[霞鹜文楷 / Space Grotesk]
生成你的第一张 AI 壁纸
```

### 设置页标签
```
[Inter / Noto Sans SC]
API 密钥设置    通用设置    壁纸设置
```

### API Key 显示
```
[Cascadia Code]
sk-proj-xxxxxxxxxxxxxxxxxxxxxxxx
```

---

## 四、字体文件清单

| 字体 | 文件名 | 大小 | 来源 |
|------|--------|------|------|
| 得意黑 | SmileySans-Oblique.ttf | ~1.8MB | GitHub: atelierAnchor/smiley-sans |
| 霞鹜文楷 | LXGWWenKai-Regular.ttf | ~4.5MB | GitHub: lxgw/LxgwWenKai |
| Space Grotesk | SpaceGrotesk-*.woff2 | ~90KB | Google Fonts |
| Inter | Inter-*.woff2 | ~60KB | Google Fonts |
| Cascadia Code | CascadiaCode-*.ttf | ~1.2MB | GitHub: microsoft/cascadia-code |

---

## 五、Logo 使用规范

### 安全空间
- Logo 周围最小留白 = 符号高度的 25%
- 不得在安全空间内放置其他元素

### 最小尺寸
- 符号版本: 16x16px (系统托盘)
- 完整 Logo: 120px 宽度 (关于页、设置页)

### 禁止行为
- 拉伸变形
- 旋转超过 15°
- 更改渐变颜色
- 添加阴影/描边效果
- 在复杂背景上不使用反白版本
