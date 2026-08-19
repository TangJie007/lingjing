# 灵镜 LINGJING — 动态壁纸

基于 **Tauri 2 + Vue 3 + TypeScript** 的动态壁纸桌面应用。

## 技术栈

- **Tauri 2** — 桌面壳（Rust 后端）
- **Vue 3** `<script setup>` + Vite
- **TypeScript**

## 开发

```bash
pnpm install        # 安装依赖
pnpm tauri dev      # 启动桌面开发模式
```

## 构建

```bash
pnpm tauri build    # 打包发行版
```

## 目录结构

```
├── src/            # 前端（Vue 3）
├── src-tauri/      # 后端（Rust / Tauri 2）
└── public/         # 静态资源
```
