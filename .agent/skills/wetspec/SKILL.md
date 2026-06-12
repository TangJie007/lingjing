---
name: wetspec
description: >
  This skill should be used when the user wants to parse a PRD (Product Requirements Document)
  into structured Spec files, or when a PRD has been updated and the corresponding Spec files
  need incremental updates. Supports .md, .pdf, and MCP-fetched documents. Generates YAML +
  Markdown dual-format Specs organized by module/folder and feature/file, with Chinese file
  naming. Triggers on requests like "解析 PRD", "把需求文档转成 spec", "PRD 变了帮我更新 spec",
  "对比新旧 PRD 差异", "增量更新需求文档", "crunch this PRD into specs", "更新用户管理模块的需求",
  "wetspec doctor", "检查 spec 覆盖率", "同步 spec 文档",
  "wetspec design", "wetspec 设计", "wetspec 技术方案",
  "wetspec build", "wetspec 写代码", "wetspec 实现", "wetspec verify", "wetspec 验收",
  "wetspec archive", "wetspec 归档", "wetspec change", "wetspec preflight".
---

# PRD-to-Spec (wetspec) — 需求文档结构化解析引擎

## Overview

将 PRD 解析为可维护的 Spec 文件（YAML + Markdown 双格式，按模块/功能组织）。
PRD 变更时**只更新受影响的 Spec**（增量模式），避免全量重生成带来的噪音。

内置完整链路：**Spec → Design → Build → Verify → Archive**，**不依赖 Comet / OpenSpec CLI**。
Comet 仅为编写本 skill 时的参考资料，不是运行时依赖。

## 前置条件（分仓：先 CLI，后 Skill）

本 Skill 与 **wetspec-cli** 分仓发布。Skill 目录**仅含**编排文档与模板，**不含** `scripts/`。

使用前必须在**业务项目**安装 CLI（npm 或 pnpm 均可）：

```bash
npm install @wetspace/wetspec-cli
# 或
pnpm add @wetspace/wetspec-cli

# 验证
npx wetspec --help
# 或
pnpm exec wetspec --help
```

安装本 Skill（Cursor）：

```bash
npx skills add https://github.com/TangJie007/ai-native-base/tree/main/.agents/skills/wetspec \
  --skill prd-to-spec -a cursor -y
```

所有确定性命令调用 `wetspec <cmd>`，禁止硬编码 `node scripts/...` 或假设 skill 内含脚本。
详见 skill 仓库 `README.md` 与 npm 包 `@wetspace/wetspec-cli` README。

### CLI 就绪检查（Guard，每次工作流入口必做）

用户可能**只安装了 Skill、未安装 CLI**。在**任何** wetspec 工作流（解析 PRD、design、build、verify、doctor 等）执行确定性命令**之前**，必须先检测 CLI：

```bash
# 检测本地依赖（按项目包管理器二选一）
npm ls @wetspace/wetspec-cli --depth=0
# 或
pnpm list @wetspace/wetspec-cli --depth=0

# 确认命令可用
npx wetspec --help
# 或
pnpm exec wetspec --help
```

包管理器推断：存在 `pnpm-lock.yaml` 时优先提示 `pnpm`；存在 `package-lock.json` 或仅 `package.json` 时提示 `npm`；不确定时两种都给出。

**判定**：

| 结果 | Agent 动作 |
|------|------------|
| `npm ls` / `pnpm list` 成功，或 `wetspec --help` 正常 | 继续工作流 |
| 命令失败、`empty`、找不到包 | **[HARD STOP]**，见下方提示 |

**未安装时必须停止**，向用户输出（不要悄悄继续、不要假设已安装）：

```
⚠️ 未检测到 @wetspace/wetspec-cli

本 Skill 只提供工作流编排，命令行工具需单独安装。请先在本项目根目录执行：

  npm install @wetspace/wetspec-cli
  # 或（pnpm 项目）
  pnpm add @wetspace/wetspec-cli

验证安装：

  npx wetspec --help
  # 或
  pnpm exec wetspec --help

若项目尚无 package.json：
  npm init -y && npm install @wetspace/wetspec-cli
  # 或
  pnpm init && pnpm add @wetspace/wetspec-cli

安装完成并确认 --help 正常后，请重新发起 wetspec 任务。
```

**禁止**在未安装时调用 `wetspec validate`、`wetspec compare` 等（必然失败）。
**禁止**未经用户同意擅自执行 `npm install` / `pnpm add`；仅**提示**用户安装，由用户确认后再装。

若用户已全局安装（`npm install -g @wetspace/wetspec-cli` 或 `pnpm add -g @wetspace/wetspec-cli`）且 `wetspec --help` 可用，可继续，但应建议项目在 `package.json` 中加入 `@wetspace/wetspec-cli` 依赖以便协作一致。

### Python 插件就绪检查（Py Guard，CLI 通过后必做）

CLI 就绪后、执行 compare/ingest/coverage 等命令**之前**，检测 Python 插件：

```bash
wetspec py-install --check
# Agent 可读 JSON：wetspec py-install --check --json
```

| `--check` 结果 | Agent 动作 |
|----------------|------------|
| 退出 0（核心依赖就绪） | 继续工作流；可选包（pymupdf/docx）缺失仅提示，不阻塞 |
| 退出非 0 | **[阻塞 AskQuestion]** → [DP-0a](references/decision_points.md#dp-0apython-插件就绪工作流入口) |

**必须用 AskQuestion**（不得静默 `py-install`、不得默认跳过）。选项见 DP-0a：

| 用户选择 | 后续 |
|----------|------|
| 安装 Python 插件 | 执行 `wetspec py-install`，成功后再继续 |
| 跳过，Node 回退 | `compare` / `coverage` 加 `--node-only`；**禁止**本轮跑 PDF/Word `ingest` |
| 暂停 | 结束本轮，提示用户装好 Python 后重试 |

**强依赖 Py 的任务**（用户选「跳过」时须 HARD STOP）：

- `wetspec ingest` 处理 `.pdf` / `.docx`

**不依赖 Py 的任务**（用户选「跳过」可正常继续）：

- `validate` / `verify` / `sync-md` / `state` / `doctor` / `build` / `design`

**混合引擎**：Node 主链 + Python 插件（路径 `node_modules/@wetspace/wetspec-cli/scripts/py/`）。`compare` / `coverage` 默认优先 Python，失败或无插件时回退 Node（`--node-only`）；`ingest` PDF/Word 无 Python 时不可用。

## 能力全景（wetspec 自包含）

| 阶段 | 命令 / 脚本 | 产出 |
|------|-------------|------|
| PRD → Spec | compare / init / validate | `specs/` |
| 增量隔离 | `wetspec_change` / `wetspec_archive` | `wetspec-delta/` |
| 技术设计 | `/wetspec-design` | `proposal.md` `design.md` `tasks.md` |
| 实现 | `/wetspec-build` | `src/<module-slug>/` + `__tests__/` |
| 验收 | `/wetspec-verify` | Spec YAML `acceptance_criteria[].verify_status` |

详见 `references/design_workflow.md`。

## 附录：wetspec vs Comet（参考对照）

| 能力 | Comet（参考） | wetspec（内置） |
|------|---------------|-----------------|
| 技术设计 | `comet-design` | `/wetspec-design` |
| 实现 | `comet-build` | `/wetspec-build` |
| 验收 | `comet-verify` | `/wetspec-verify` |
| PRD 结构化解析 | ❌ | ✅ 核心能力 |
| PRD↔Spec 覆盖率 | ❌ | ✅ `check_coverage.js` |
| 中文模块/功能命名 | ❌ | ✅ |

---

## Workflow Decision Tree

```
用户请求
    │
    ├── 首次解析 PRD → [全量解析工作流]（仅主 specs/ 为空时）
    ├── PRD 更新 + 有 wetspec change → [Change 增量工作流]（**默认，多人推荐**）
    ├── PRD 更新 + 无 change → [主库增量工作流]（单人/小团队）
    │       └── 结构大变 / 增量不可靠 → [全量同步工作流]（fallback）
    ├── /wetspec-archive → [delta 回写主 specs]
    ├── wetspec preflight → 多人协作预检
    ├── phase=specs-ready → [设计阶段决策点]（阻塞，必须 AskQuestion）
    ├── /wetspec-design → [设计阶段工作流]（wetspec 内置）
    ├── phase=design → [设计阶段工作流]（续作）
    ├── /wetspec-build → [实现工作流]（按 Spec + design 写代码）
    ├── /wetspec-verify → [验收工作流]（按 AC 自动+人工验收）
    ├── 校验 Spec → validate_spec.js
    ├── 健康诊断 → wetspec_doctor.js
    ├── 检查 MD 漂移 → sync_spec_md.js --check
    └── 检查 PRD 覆盖率 → check_coverage.js
```

### 阶段流转

```
parse / update / sync
        ↓
   specs-ready  ──[DP-1 AskQuestion 阻塞]──→  /wetspec-design
        │                                              ↓
        │                                         design（写 proposal/design/tasks）
        │                                              ↓
        │                                    [DP-3 AskQuestion 阻塞]  ← 不可跳过
        │                                              ↓
        └──────── skip-design / build ──────────→  /wetspec-build
                                                              ↓
                                                    build → verify → done
```

**核心规则**：

- Spec 生成/更新完成后，**不得**直接结束会话；必须先进入 `specs-ready` 决策点。
- 遇到决策点，**必须使用 AskQuestion 工具暂停并等待用户明确选择**；不得用默认推荐、历史偏好或纯文字提示代替。
- 用户选择「进入设计」后，执行 `/wetspec-design`（`start-design`）；选择「暂不设计」则 `skip-design` → `done`。
- 若 `auto_transition: false`，完成 Spec 后只提示 `/wetspec-design` 或 `/wetspec-build`，不自动跳转。
- **禁止因端到端指令跳过决策点**：用户说「完成需求」「一次性做完」「/wetspec 完成需求」等，**只表示目标**，**不代替** DP-1 / DP-3 的 AskQuestion。详见 [decision_points.md §禁止因端到端指令跳过](references/decision_points.md#禁止因端到端指令跳过决策点hard-stop)。

**阻塞决策点**（到达时必须 AskQuestion，详见 `references/decision_points.md`）：

0. **DP-0a Python 插件**（CLI 就绪后；`wetspec py-install --check` 失败时）— 见 [DP-0a](references/decision_points.md#dp-0apython-插件就绪工作流入口)
1. **DP-0 单元测试框架**（首次 `wetspec init` 后；`unit_test.framework` 未配置时）— 见 [DP-0](references/decision_points.md#dp-0单元测试框架项目首次初始化)
2. **DP-1 Spec / delta 分析完成**（`specs-ready`）— 全量、增量、**Change 工作流 validate-delta 后**
3. **DP-2 增量 vs 全量同步**（结构大变时）
4. **DP-3 设计产物确认**（`design-complete` 前）
5. **DP-4 验收失败**（verify fail）
6. **DP-5 archive 预览后**（dry-run 后，推荐阻塞）

**分析跑完后的固定动作**：先展示摘要 → **AskQuestion** → 用户选定后再执行 archive / design / build。**禁止**分析完直接给文字建议就结束。

**AskQuestion 选项文案**：每个选项 `label` 必须带括号说明，格式 `动作（选了这个会怎样）`，见 `references/decision_points.md`。

**红旗自检**（出现这些想法时停下）：

| Agent 想法 | 实际风险 |
|-----------|---------|
| "用户大概想直接设计" | 不能替用户决定 — 必须 AskQuestion |
| "用户说完成需求，我帮他全跑完" | **严禁** — 仍须 DP-1 →（若进设计）DP-3，见 decision_points §端到端 |
| "design.md 写完了，顺手把代码也写了" | **严禁** — DP-3 未确认前不得写 `src/` |
| "改动很小，不用问" | 决策点无大小例外 |
| "上次选了设计，这次也选" | 历史偏好不能代替当前确认 |
| "auto_transition 是 true，可以自动往下走" | `auto_transition` **不豁免** AskQuestion |

---

## 全量解析工作流（PRD → Specs）

Trigger: 用户提供 PRD，且 `specs/` 不存在或为空。

### Step 0: 初始化状态（Guard 入口）

```bash
wetspec state check --transition start-parse
wetspec state set specs/ --field phase --value parse
```

### Step 1–2: 读取 PRD，解析模块/功能结构

支持 `.md` / `.txt` / MCP 文档；PDF 需先转文本。

提取：模块、功能、描述、用户故事、验收标准、API、依赖、NFR。

### Step 3: 生成 `.modules.json`

```json
{
  "modules": [{
    "name": "用户登录",
    "features": [{
      "id": "LOG-001",
      "name": "手机号+验证码登录",
      "description": "...",
      "priority": "high",
      "acceptance": ["..."]
    }]
  }]
}
```

也支持 skill 文档中的数组格式 `[{ "name": "...", "features": [...] }]`。

ID 格式：`{模块缩写}-{3位序号}`，见 `references/spec_schema.md`。

### Step 4: 初始化目录结构

```bash
wetspec init specs/.modules.json specs/ 1.0.0
```

自动创建 Spec 骨架、**README.md**、**INDEX.md**、**.wetspec.yaml**。

### Step 4.5: 单元测试框架（DP-0，阻塞）

**触发**：`wetspec state get specs/ --field unit_test` 为 `null` 或 `unit_test.framework` 为空。

**已配置则跳过**（老项目已有 `unit_test.framework`）。

1. 检测并展示推荐：

```bash
wetspec unit-test detect --root . --json
```

2. **必须用 AskQuestion**（推荐项放第一；选项含括号说明，见 [DP-0](references/decision_points.md#dp-0单元测试框架项目首次初始化)）。

3. 用户选定后：

- **`node:test`**：`check` 通过 → `configure`（无需安装）
- **`vitest` / `jest` / `pytest`**：`await` → 展示 `installCommands` → **硬停**（`phase=awaiting-unit-test`）→ 用户本机安装 → 用户说「继续」→ `check` 通过 → `configure`
- **禁止** Agent 执行 `npm install` / `pnpm add` / `configure --install`

```bash
wetspec unit-test detect --root . --json
wetspec unit-test await specs/ --framework <id> --root .    # 需安装的框架
wetspec unit-test check --framework <id> --spec-dir specs/ --root .
wetspec unit-test configure specs/ --framework <id> --root .
```

**单层测试说明**（须在 AskQuestion 前告知用户）：

| 用途 | 路径 | 执行 |
|------|------|------|
| 实现 + 验收 | `src/<module-slug>/__tests__/` | build：`unit_test.command`；verify：按 AC 嵌套 describe 逐条跑 |

验收测试嵌套约定（`node:test`）：`describe('<feature_id>')` → `describe('AC-001: ...')` → `it(...)`。`wetspec verify` 用 `--test-name-pattern "<feature_id> AC-001"` 匹配。

### Step 5: 填充 Spec 内容

- **YAML**：基于 `assets/spec_template.yaml`（metadata、user_stories、acceptance_criteria、changelog…）
- **Markdown**：基于 `assets/spec_template.md`，或 Step 6 用脚本生成

规则：
- `changelog` 至少一条初始记录（version: "1.0.0", type: "added"）
- `metadata.version` = "1.0.0"，`metadata.status` = "draft"
- PRD 未提及的字段标注「待补充」，不要编造

### Step 6: 同步、校验、进入决策点

```bash
wetspec sync-md specs/          # YAML → MD 同步
wetspec validate specs/
wetspec coverage PRD.md specs/ # PRD 覆盖率
wetspec state set specs/ --field prd.current --value PRD.md
wetspec state check specs/ --transition parse-complete
wetspec state set specs/ --field phase --value specs-ready
```

删除临时 `.modules.json`。

### Step 7: 设计阶段决策点（阻塞）

向用户展示 Spec 摘要（模块数、功能数、覆盖率、受影响文件），然后 **必须用 AskQuestion** 询问：

> Spec 已生成完毕。是否进入技术设计阶段？

选项（AskQuestion `label` 须含括号说明，见 [DP-1b](references/decision_points.md#dp-1b全量解析完成step-7-专用选项略少)）：

| 选项 label | 动作 |
|------------|------|
| 进入技术设计（生成 proposal/design/tasks 技术方案，不写代码） | `/wetspec-design` |
| 暂不设计，到此为止（Spec 已就绪，实现时再 build；可跑 doctor） | `skip-design` → `done` |
| 先 review / 修改 Spec（人工改 YAML/MD 后重新 validate，不执行写库或写代码） | 保持 `specs-ready` |
| 跳过设计，直接实现（Spec 已够清晰，立即写 src 与 AC 测试） | `/wetspec-build` |

**不得**跳过此步骤直接结束。即使用户表述为「完成需求」「一次性做完」，也**必须先**完成本步 AskQuestion；「完成需求」不是已选「直接实现」的授权，只有用户在本步点选 `build` 选项才算。

---

## Change 隔离工作流（多人协作默认）

> **核心原则**：change 内只存 `affected_specs` 的 delta，**禁止**全量复制主 `specs/`；archive 时才回写主库。

详见 `references/change_isolation.md`。

### 目录约定

```
specs/                              ← 主库（canonical，每功能一份）
<changes_root>/<name>/              ← 默认 wetspec/changes/<name>/
  .wetspec.yaml                     ← scope: change
  proposal.md / design.md / tasks.md  ← wetspec 设计产物
  wetspec-delta/                    ← 仅 affected_specs 的 YAML
    MANIFEST.json
    用户登录/手机号+验证码登录_spec.yaml
```

`changes_root` 在 `specs/.wetspec.yaml` 配置，**默认 `wetspec/changes`**。旧仓库若 change 仍在 `openspec/changes/`，可保留该配置直至迁移完成。

### Step C0: 预检（preflight）

```bash
wetspec preflight wetspec/changes/<name> \
  --main-specs specs/ --prd PRD_new.md
```

检查：主库 `phase` 非忙碌、change 未归档、协作规则提示。

### Step C1: 初始化 change 工作区

```bash
wetspec change init wetspec/changes/<name> \
  --main-specs specs/ --change-name <name> --prd PRD_new.md
```

### Step C2: PRD diff + MANIFEST

```bash
wetspec compare PRD_old.md PRD_new.md \
  --output wetspec/changes/<name>/diff.json --spec-dir specs/
wetspec change set-manifest wetspec/changes/<name> \
  --diff wetspec/changes/<name>/diff.json

# compare_prd 未映射到 Spec 时（如正文变更），AI 语义补充后手动指定：
wetspec change set-manifest wetspec/changes/<name> \
  --diff diff.json --affected-file wetspec/changes/<name>/affected.json
```

### Step C3: AI 只写 delta（Guard）

对 `MANIFEST.affected_specs` 中每个路径：

- **写入位置**：`<changes_root>/<name>/wetspec-delta/<path>`（不是主 `specs/`）
- **禁止**：复制整个 `specs/用户登录/` 或其它未 affected 的文件

```bash
wetspec change validate-delta wetspec/changes/<name>
```

`validate-delta` 失败 → **[HARD STOP]**，删除非法文件或修正 MANIFEST。

### Step C3.5: 决策点（阻塞，必须 AskQuestion）

delta 校验通过后：

```bash
wetspec state set specs/ --field phase --value specs-ready
wetspec state set specs/ --field prd.current --value <新PRD>
wetspec state set specs/ --field active_change --value <change-name>
```

向用户展示：PRD 差异摘要、`affected_specs`、delta 文件列表、`check_coverage` 结果。

**必须用 AskQuestion**（不得跳过），选项见 [DP-1](references/decision_points.md#dp-1spec--delta-就绪最常用)：

| 选项 label（含括号） | 动作 |
|----------------------|------|
| 进入技术设计（生成技术方案文档，明确配置与 AC 映射） | `/wetspec-design` |
| 归档 delta 到主 specs（delta 回写主库，主库成为 Spec 真相源） | dry-run → DP-5 → archive |
| 直接实现（跳过设计，按 Spec/delta 写代码与测试） | `/wetspec-build` |
| 先 review / 修改（只检查 delta，不写库不写代码） | 保持 `specs-ready` |
| 暂停，稍后继续（保留 change，下次从同一路径恢复） | 保持 `specs-ready` |

### Step C4: 设计 / 实现（用户选择后）

- 设计产物在 `<changes_root>/<name>/`（proposal、design、tasks）
- `/wetspec-build` 可读主 specs + delta 合并理解，或 archive 后再实现

### Step C5: Archive 回写主 specs

```bash
wetspec archive wetspec/changes/<name> --dry-run
wetspec archive wetspec/changes/<name>
```

脚本行为：仅将 delta 中 `affected_specs` 复制到主 `specs/` → `sync_spec_md` → `generate_indexes` → 更新主 `.wetspec.yaml`。

### 多人冲突策略

| 场景 | 处理 |
|------|------|
| A、B 各改不同功能 | 各自 change，各自 archive，无冲突 |
| A、B 改同一功能 | Git PR 冲突；以最新 PRD 重跑 compare_prd，**不要手 merge changelog** |
| 有人直接改主 specs | preflight 告警；应走 change + archive |

---

## 增量更新工作流（PRD 变更 → 增量更新 Specs）

Trigger: 新版 PRD + 已有 `specs/`。

<IMPORTANT>
**有活跃 wetspec change 时，必须走 [Change 隔离工作流](#change-隔离工作流多人协作默认)，不得直接改主 `specs/`。**
以下主库增量流程仅用于：无 change、单人维护、或首次建仓后的直接更新。
</IMPORTANT>

**核心原则**：只更新 `affected_specs`，不碰其它文件。

### Step 1: 确认旧版 PRD

从用户提供或 `metadata.prd_source` / `.wetspec.yaml` 的 `prd.previous` 推断。

### Step 2: 差异对比（含 affectedSpecFile 映射）

```bash
wetspec compare old_prd.md new_prd.md \
  --output diff.json --spec-dir specs/
```

输出：`summary`、`details`（含 modified 检测）、`affected_specs`、`unified_diff`。

### Step 3: Guard + 状态

```bash
wetspec state check --transition start-update
wetspec state set specs/ --field last_diff --value diff.json
wetspec state set specs/ --field affected_specs --value '["用户登录/xxx_spec.yaml"]'
```

### Step 4: AI 语义分析 + 增量更新

对每个 `affected_specs` 中的文件：

| 变更类型 | 动作 |
|----------|------|
| 新增功能 | 创建 YAML+MD，更新 INDEX.md |
| 修改功能 | 只改变化字段，bump version，追加 changelog |
| 删除功能 | `metadata.status` = deprecated，不删文件 |

版本策略：patch=文案修复，minor=新 AC/故事，major=破坏性 API 变更。

### Step 5: 同步 MD + 校验 + 决策点

```bash
wetspec sync-md specs/ --file specs/模块/功能_spec.yaml  # 或全目录
wetspec indexes specs/ --prd new_prd.md
wetspec validate specs/
wetspec state check specs/ --transition update-complete
wetspec state set specs/ --field phase --value specs-ready
```

然后执行 [Step 7: 设计阶段决策点](#step-7-设计阶段决策点阻塞)（与全量解析相同）。

---

## 全量同步工作流（archive 式 fallback）

Trigger: PRD 结构大幅重组、增量对比不可靠、或用户明确要求全量对齐。

```bash
# 预览（dry-run，不写入）
wetspec sync new_prd.md specs/ --dry-run

# 执行：重建索引 + 同步 MD + 更新状态
wetspec sync new_prd.md specs/
```

全量同步后，AI 仍需根据 PRD 更新各 YAML 内容；脚本负责索引、MD、状态一致性。

同步完成后：

```bash
wetspec state check specs/ --transition sync-complete
wetspec state set specs/ --field phase --value specs-ready
```

然后执行 [Step 7: 设计阶段决策点](#step-7-设计阶段决策点阻塞)。

---

## 设计阶段工作流（/wetspec-design）

Trigger: `/wetspec-design`、`wetspec 设计`；或 `specs-ready` 决策点选择「进入技术设计」；或 `phase=design` 续作。

wetspec 负责 **WHAT**（`specs/`）与 **HOW**（`design.md` / `tasks.md`），全程使用 wetspec 命令，**不调用 Comet / OpenSpec CLI**。

完整步骤见 `.cursor/commands/wetspec-design.md` 与 `references/design_workflow.md`。

### Step D1: Guard + 状态

```bash
wetspec state check specs/ --transition start-design
wetspec state set specs/ --field phase --value design
wetspec state set specs/ --field active_change --value <change-name>
wetspec state set specs/ --field changes_root --value wetspec/changes
```

### Step D2: 创建或续接 wetspec Change

1. 若 PRD 增量已执行过 [Change 隔离工作流](#change-隔离工作流多人协作默认)，复用同一 change 目录
2. 若无 change，从 PRD + `specs/` 推导 kebab-case 名称：

```bash
wetspec change init <changes_root>/<name> \
  --main-specs specs/ --change-name <name> --prd <PRD文件>
```

### Step D3: 从 Spec 生成设计产物

读取 `specs/`（及 `wetspec-delta/` 如有），在 `<changes_root>/<name>/` 生成：

| 产物 | 模板 |
|------|------|
| `proposal.md` | `assets/design_proposal_template.md` |
| `design.md` | `assets/design_template.md` |
| `tasks.md` | `assets/tasks_template.md` |

`design.md` 必须包含：**配置常量表**、**AC→测试映射**，供 `/wetspec-build` 直接引用。

生成后 **必须 AskQuestion 确认（DP-3，阻塞）**：

> **硬停**：`proposal.md` / `design.md` / `tasks.md` 落盘后 **禁止** 同轮会话内继续写 `src/` 或执行 `start-build`。
> 须先展示设计摘要，调用 AskQuestion，**等待用户点选** 后再进入 build。
> 用户说「完成需求」**不能**代替本步确认。

| 选项 label（含括号） | 动作 |
|----------------------|------|
| 确认设计，进入实现（认可 design.md，之后可 /wetspec-build） | `design-complete` → 提示 build |
| 先归档 Spec 再实现（主库先更新，再按主库 Spec 写代码） | archive → build |
| 需要修改设计（改 proposal/design/tasks，不重回 Spec 解析） | 保持 `phase=design` |
| 暂停（设计已生成未确认，下次从 design 续作） | 保持 `phase=design` |

```bash
wetspec state set specs/ --field design_doc --value <changes_root>/<name>/design.md
wetspec state check specs/ --transition design-complete
wetspec state set specs/ --field phase --value done
```

### 恢复规则

1. 读取 `specs/.wetspec.yaml` 的 `phase`
2. `specs-ready` → AskQuestion，不假设已选设计
3. `design` + `active_change` → 检查 `proposal.md` / `design.md` / `tasks.md`，从缺失处续作

---

## 实现工作流（/wetspec-build）

Trigger: `/wetspec-build`、`wetspec 写代码`、`wetspec 实现 <功能ID>`，或设计完成后用户选择直接实现。

### Step B1: 定位 Spec

- 用户提供路径 / ID（如 `LOG-001`）→ 在 `specs/` 查找对应 YAML
- 否则读 `build_target` 或 `affected_specs[0]`

### Step B2: Guard + 状态

若 `unit_test` 未配置或 `unit_test.deferred: true`，**必须先执行 DP-0**（`wetspec unit-test configure`），不得开始写代码。

```bash
wetspec state check specs/ --transition start-build
wetspec state set specs/ --field phase --value build
wetspec state set specs/ --field build_target --value <相对路径>
```

### Step B3: 读取上下文并实现

必读：Spec YAML 的 `acceptance_criteria`、`design_doc`（`/wetspec-design` 产出）、同 change 下 `tasks.md`。

代码约定：

| 路径 | 用途 |
|------|------|
| `src/<module-slug>/` | 功能实现（**英文 kebab-case，禁止中文路径**） |
| `src/<module-slug>/__tests__/` | 单元测试 + AC 验收（同一套测试） |

**模块 slug 映射**：`design.md` 必须写明 PRD 中文模块 → 英文目录，例如 `用户登录` → `user-login`。Spec 仍用 `specs/用户登录/`。

实现规则：
- `test_method: auto/both` 的 AC **必须**在 `__tests__/` 中有 `describe('<feature_id>')` → `describe('AC-xxx: ...')` 嵌套
- 配置与设计一致（如 `SMS_CODE_TTL_SECONDS=600`）
- **不要**再创建 `tests/<feature_id>/` 或 `reports/` 目录

### Step B3.5: 单元测试（含 AC 覆盖）

在 `src/<module-slug>/__tests__/` 按 `unit_test.framework` 编写测试；每个 auto AC 用 `LOG-xxx` / `AC-xxx` 嵌套 describe 标出。

build 完成前必须跑通：

```bash
# 读取 specs/.wetspec.yaml → unit_test.command，通常为：
npm run test:unit
```

### Step B4: 进入验收

```bash
wetspec state check specs/ --transition build-complete
wetspec state set specs/ --field phase --value verify
```

然后执行 [验收工作流](#验收工作流-wetspec-verify)。

---

## 验收工作流（/wetspec-verify）

Trigger: `/wetspec-verify`、`wetspec 验收`，或 build 完成后自动衔接。

### Step V1: 运行验收脚本

```bash
wetspec verify <spec_yaml> --root <project_root>
```

脚本行为：
1. 读取 `acceptance_criteria`（及 legacy `spec.acceptance` 作人工清单）
2. 对 `test_method: auto/both` 的 AC，在 `unit_test.path` 下按 `describe('<feature_id>')` → `describe('AC-xxx')` 跑单元测试
3. 将 `verify_status` / `verified_at` / `verify_reason` 写回 Spec YAML（默认开启，`--no-write` 可只读）
4. 终端输出逐项 pass/fail/manual

### Step V2: 更新状态

**通过**（`verify_result: pass`）：

```bash
wetspec state set specs/ --field verify_result --value pass
wetspec state check specs/ --transition verify-pass
wetspec state set specs/ --field phase --value done
```

`wetspec verify` 已将 Spec `metadata.status` 更新为 `implemented`（验收通过时）。

**失败**：

```bash
wetspec state set specs/ --field verify_result --value fail
wetspec state check specs/ --transition verify-fail
wetspec state set specs/ --field phase --value build
```

**必须用 AskQuestion**（选项含括号说明，见 [DP-4](references/decision_points.md#dp-4验收失败)）：

| 选项 label | 动作 |
|------------|------|
| 修复代码后重试（改 src/__tests__ 使 AC 通过） | `verify-fail` → build |
| Spec 有误，先改 Spec（AC 或需求写错，先更新 YAML） | 回到 `specs-ready` |
| 接受偏差并记录（业务认可不满足 AC，文档记录原因） | 人工 sign-off |

### 验收报告示例

```
=== wetspec 验收报告: 手机号+验证码登录 (LOG-001) ===

✅ AC-001 [both] 发送6位验证码 TTL 600s
✅ AC-002 [both] 过期提示
✅ AC-003 [both] 60秒频率限制
⏸️ LEGACY-001 [manual] 需人工验收

汇总: 自动通过 5/5，失败 0，待人工 5
验收结论: PASS
```

---

## wetspec-cli（独立 npm 包，与 Skill 分仓）

CLI 发布为 npm 包 `@wetspace/wetspec-cli`，与 Skill 仓库分离。业务项目安装后任意终端 / CI / Agent 均可调用：

```bash
npm install @wetspace/wetspec-cli   # 或 pnpm add @wetspace/wetspec-cli
wetspec --help

wetspec compare PRD_old.md PRD_new.md --spec-dir specs/ -o diff.json
wetspec change validate-delta wetspec/changes/<name>
wetspec archive wetspec/changes/<name> --dry-run
wetspec doctor specs/
```

Skill 目录**无** `package.json`、**无** `scripts/`；所有命令走已安装的 `wetspec` CLI。
详见 <https://github.com/TangJie007/ai-native-base/tree/main/wetspec-cli>（npm: `@wetspace/wetspec-cli`）。

> **编排**（AskQuestion、模板、工作流）在 Skill 仓库；**确定性逻辑**在 CLI npm 包。

---

## Python 插件（CLI + Py 增强）

安装一次：

```bash
wetspec py-install
# 仅检测: wetspec py-install --check
# 业务项目 package.json 可加: "py:install": "wetspec py-install"
```

| 优先级 | Python 脚本 | Node 入口 | 能力 |
|--------|-------------|-----------|------|
| P0 | `py/prd_ingest.py` | `prd_ingest.js` | PDF/Word → Markdown |
| P1 | `py/map_affected.py` | `compare_prd.js` / `wetspec_change.js set-manifest` | 正文级 affected 映射 |
| P2 | `py/check_coverage.py` | `check_coverage.js`（默认） | rapidfuzz 模糊覆盖率 |
| P3 | `py/compare_prd.py` | `compare_prd.js`（默认） | 章节+正文 diff |

路由规则：

- `compare_prd.js` / `check_coverage.js` **默认优先 Python**，失败或无 Python 时回退 Node
- 强制 Node：`--node-only`
- `wetspec_change.js set-manifest` 在 `affected_specs` 为空时自动调用 `map_affected.py`

```bash
# P0: PDF → Markdown
wetspec ingest PRD.pdf --output PRD.md

# P3: 增强 diff（自动映射 LOG-001 等）
wetspec compare old.md new.md --spec-dir specs/ -o diff.json

# P1: 单独补强 affected（路径随 npm 包）
python node_modules/@wetspace/wetspec-cli/scripts/py/map_affected.py --diff diff.json --spec-dir specs/ -o diff.json
```

---

## 日常维护命令

均在**业务项目根**执行（已 `npm install @wetspace/wetspec-cli` 或 `pnpm add @wetspace/wetspec-cli` 后可用 `wetspec`）。

| 命令 | 用途 |
|------|------|
| `wetspec --help` | CLI 命令列表 |
| `wetspec py-install` | 安装 Python 插件依赖 |
| `wetspec py-install --check` | 检测 Python 与依赖是否就绪 |
| `wetspec ingest <file> [--output md]` | PRD 文档摄入 |
| `wetspec preflight <change> --main-specs specs/` | 多人协作预检 |
| `wetspec change init/set-manifest/validate-delta` | Change delta 管理 |
| `wetspec archive <change> [--dry-run]` | delta → 主 specs |
| `wetspec verify <spec_yaml> --root .` | 按 AC 验收实现 |
| `wetspec doctor specs/` | 健康诊断（含 unit_test 是否已配置） |
| `wetspec unit-test detect [--root .]` | 识别项目类型，推荐框架与 installCommands（DP-0） |
| `wetspec unit-test check [--framework <id>]` | 检查用户是否已安装依赖（退出 0=可继续） |
| `wetspec unit-test await specs/ --framework <id>` | 暂停流程，等待用户自装 |
| `wetspec unit-test configure specs/ --framework <id>` | 写入 unit_test + test:unit 脚本（须 check 通过） |
| `wetspec sync-md specs/ --check` | 检查 MD 是否与 YAML 一致 |
| `wetspec sync-md specs/` | 从 YAML 重新生成 MD |
| `wetspec coverage PRD.md specs/` | PRD 功能是否都有 Spec |
| `wetspec state get specs/` | 查看当前工作流状态 |
| `wetspec indexes specs/ --prd PRD.md` | 重建 README + INDEX |

---

## 关键设计约束

### 文件命名
- **Spec 模块目录**：PRD 中文模块名（如 `specs/用户登录/`）
- **代码目录**：英文 `module-slug`（kebab-case，如 `src/user-login/`），**禁止中文**
- **功能 Spec 文件**：`{功能名}_spec.yaml` / `{功能名}_spec.md`（中文功能名）
- **代码文件**：英文 kebab-case（如 `sms-login.js`）

### YAML 格式
- **标准格式**（推荐）：顶层 `metadata` + `description` + `user_stories` + …
- **legacy 块**：可选 `spec:` 块，供旧工具兼容

### 增量 vs 全量选择

| 场景 | 推荐模式 |
|------|----------|
| 单个功能 AC 变更 | incremental |
| 新增 1–2 个功能 | incremental |
| 模块拆分/合并/重命名 | full sync |
| doctor 报告覆盖率 < 100% 且 PRD 已大改 | full sync |

### 内容提取精度
- 优先从 PRD 原文提取
- 未提及标注「待补充」
- 用户故事格式化为「作为…我希望…以便…」

---

## Resources

### wetspec-cli（npm 包，脚本唯一维护位置）

仓库：<https://github.com/TangJie007/ai-native-base/tree/main/wetspec-cli> · 安装：`npm install @wetspace/wetspec-cli` 或 `pnpm add @wetspace/wetspec-cli`

| CLI 命令 | 说明 |
|----------|------|
| `wetspec compare` | PRD 差异对比 + affected_specs |
| `wetspec init` | 初始化目录 + 索引 |
| `wetspec validate` | 校验 YAML |
| `wetspec sync-md` | YAML → MD |
| `wetspec indexes` | README / INDEX |
| `wetspec coverage` | PRD ↔ Spec 覆盖率 |
| `wetspec state` | `.wetspec.yaml` + Guard |
| `wetspec sync` | 全量同步（--dry-run） |
| `wetspec doctor` | 健康诊断 |
| `wetspec unit-test` | 单元测试框架 detect / configure |
| `wetspec py-install` | 安装 / 检测 Python 插件 |
| `wetspec verify` | AC 验收 |
| `wetspec change` | delta 隔离 |
| `wetspec archive` | 回写主 specs |
| `wetspec preflight` | 多人预检 |
| `wetspec ingest` | PRD 摄入 |

Python 插件随 npm 包安装在 `node_modules/@wetspace/wetspec-cli/scripts/py/`。Skill 仓库**不包含** `scripts/` 副本。

### references/
- `spec_schema.md` — YAML 字段定义
- `wetspec_state_schema.md` — 状态文件 Schema
- `design_workflow.md` — 内置 design/build 工作流
- `decision_points.md` — **AskQuestion 阻塞决策点手册（Agent 必读）**
- `change_isolation.md` — Change 隔离与 archive 规则

### assets/
- `spec_template.yaml` / `spec_template.md` — Spec 模板
- `.wetspec.yaml.template` — 主库状态模板
- `.wetspec-change.yaml.template` — change 级状态模板

---

## 踩坑经验

- **禁止**把主 `specs/` 整目录复制进 change；只放 `wetspec-delta/affected_specs`
- archive 前必须 `validate-delta`；非法文件会导致 archive 拒绝
- 有活跃 change 时，**不要**两人直接改主 `specs/`，应各自 change → archive
- Spec 生成后**不要**直接设 `phase=done`；必须经过 `specs-ready` 决策点
- **禁止**因「完成需求 / 一次性做完」跳过 DP-1、DP-3；用户端到端意图 ≠ AskQuestion 已确认
- `design.md` 生成后**必须**停 DP-3，**禁止**未确认就写 `src/`
- 决策点必须用 **AskQuestion 工具**，纯文字提问不算完成
- AskQuestion 每个选项 **label 必须含（）说明**，告知用户选该项的后果
- `wetspec compare` Node 回退模式依赖标题级对比；**默认 Python 引擎**支持正文级 diff
- PDF/Word 用 `wetspec ingest`（需先 `wetspec py-install`）
- 无 Python 时加 `--node-only`，或仅使用 .md PRD
- 修改 YAML 后务必运行 `wetspec sync-md`，否则 doctor 会报 MD 漂移
- `wetspec init` 不覆盖已存在 Spec；全量重建需手动备份或删目录
- 首次 init 后**必须**走 DP-0 配置 `unit_test`；`doctor` 会对未配置项告警
- DP-0 **禁止** Agent `npm install`；vitest/jest 须 `await` → 用户自装 → 说「继续」→ `unit-test check`（宽松：任一主框架放行，配套仅警告）→ `configure`
- `node:test` 项目：`wetspec verify` 按 LOG/AC 嵌套 describe 逐条验收；其他框架整包跑 `unit_test.command` 后粗粒度标记 AC
- 只装 Skill 未装 CLI 时**必须 HARD STOP 并提示安装**，不得跳过 CLI Guard
- Py 插件未就绪时**必须 AskQuestion（DP-0a）**，禁止擅自 `wetspec py-install`；用户选跳过则 compare/coverage 用 `--node-only`
- 增量更新失败时，用 `wetspec sync --dry-run` 预览后再决定 full sync
