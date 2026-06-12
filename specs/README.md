# Spec 目录

> 由 wetspec 管理 | 功能总数：21  
> 来源 PRD：AI动态桌面_V1_PRD.md  
> 最后索引更新：2026-06-12

## 模块概览

| 模块 | 功能数 | 功能 |
|------|--------|------|
| [AI生成管线](./AI生成管线/INDEX.md) | 3 | AI意图分析与方案构建、专业壁纸生成、构建方案预览与确认 |
| [API Key管理](./API Key管理/INDEX.md) | 3 | API Key有效性检测、API Key配置与加密存储、多平台API管理 |
| [基础设置](./基础设置/INDEX.md) | 5 | API默认值与生成配置、国际化框架（i18n）、壁纸播放设置、性能与存储设置、通用设置 |
| [壁纸库管理](./壁纸库管理/INDEX.md) | 2 | AI生成历史、本地壁纸管理 |
| [壁纸播放引擎](./壁纸播放引擎/INDEX.md) | 3 | 全屏检测自动暂停、视频壁纸播放、静态图片壁纸播放 |
| [系统托盘](./系统托盘/INDEX.md) | 1 | 托盘最小化与右键菜单 |
| [首次启动向导](./首次启动向导/INDEX.md) | 4 | 快速上手与一键生成、欢迎页与品牌动画、火山引擎注册引导、跳过API配置的本地模式 |

## 常用命令

```bash
# 需已安装: npm install @wetspace/wetspec-cli 或 pnpm add @wetspace/wetspec-cli
wetspec validate specs/
wetspec sync-md specs/ --check
wetspec coverage <prd> specs/
wetspec doctor specs/
```
