# wetspec-skill

Cursor Agent Skill：PRD → Spec → Design → Build → Verify → Archive 全链路编排。

**与 [wetspec-cli](https://github.com/TangJie007/ai-native-base/tree/main/wetspec-cli) 同属 [ai-native-base](https://github.com/TangJie007/ai-native-base) monorepo**——本目录仅含文档与模板，**不含**可执行脚本。

| 仓库 | 职责 |
|------|------|
| **wetspec-skill**（本仓库） | `SKILL.md`、AskQuestion 决策点、`assets/` 模板、`references/` |
| **wetspec-cli** | `wetspec` 命令行：校验、diff、verify、doctor … |

## 安装

### 1. 安装 CLI（必需）

在**业务项目根目录**执行：

```bash
npm install @wetspace/wetspec-cli
# 或
pnpm add @wetspace/wetspec-cli

npx wetspec --help
# 或
pnpm exec wetspec --help
```

可选 Python 增强（工作流启动时会 `--check`；未就绪时 Agent 会 **AskQuestion** 询问是否安装）：

```bash
wetspec py-install --check   # 检测
wetspec py-install           # 安装（用户确认后由 Agent 执行）
```

### 2. 安装 Skill（Cursor）

```bash
# 安装到当前项目
npx skills add https://github.com/TangJie007/ai-native-base/tree/main/.agents/skills/wetspec \
  --skill prd-to-spec -a cursor -y

# 全局安装（所有项目可用）
npx skills add https://github.com/TangJie007/ai-native-base/tree/main/.agents/skills/wetspec \
  --skill prd-to-spec -a cursor -g -y
```

仓库：<https://github.com/TangJie007/ai-native-base>

> **注意**：仅安装 Skill **不会**安装 CLI。若未执行步骤 1，Agent 会提示你先运行 `npm install @wetspace/wetspec-cli` 或 `pnpm add @wetspace/wetspec-cli`，不会自动代装。

### 3. 本地路径安装（开发本 skill 时）

```bash
npx skills add ./path/to/wetspec-skill -a cursor --copy -y
```

## 仓库结构（发布用）

拆仓时建议将本目录作为 **wetspec-skill** 仓库根，或放在 `skills/prd-to-spec/` 下：

```
wetspec-skill/
├── SKILL.md           # 主指令（name: prd-to-spec）
├── README.md          # 本文件
├── references/        # 决策点、schema、工作流
└── assets/            # Spec / design 模板
```

**不要**将 `wetspec-cli/scripts/` 复制进本仓库。

## 使用

安装完成后，在 Cursor 中触发例如：

- 「解析 PRD」「把需求文档转成 spec」
- `/wetspec-design`、`/wetspec-build`、`/wetspec-verify`
- `wetspec doctor`、`wetspec archive`

Agent 会读取本 Skill 的工作流，并调用已安装的 `wetspec` CLI 执行确定性操作。

## 发布

Skill **不发 npm**。发布步骤：

1. 将本目录推送到 GitHub 仓库 `wetspec-skill`
2. 用户通过 `npx skills add TangJie007/ai-native-base`（或上述 tree 路径）安装
3. （可选）向 [skills.sh](https://skills.sh/) 生态推广

## 与 CLI 版本

Skill 文档与 CLI 命令保持同步。升级 CLI：

```bash
npm update @wetspace/wetspec-cli
# 或
pnpm update @wetspace/wetspec-cli
npx skills update   # 如有 skill 更新
```
