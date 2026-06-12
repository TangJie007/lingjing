# wetspec 设计工作流（内置，无外部依赖）

## 定位

| 阶段 | 负责方 | 产出 |
|------|--------|------|
| PRD → Spec | wetspec | `specs/` YAML + MD |
| 技术设计 | **wetspec** `/wetspec-design` | `proposal.md` `design.md` `tasks.md` |
| 实现 | **wetspec** `/wetspec-build` | `src/` `tests/` |
| 验收 | **wetspec** `/wetspec-verify` | Spec YAML `verify_status` 字段 |

Comet / OpenSpec **不是**运行时依赖；仅为 wetspec 设计状态机与工作流时的**参考借鉴**（见 `wetspec_state_schema.md` 附录）。

**分仓**：编排在本 Skill 仓库；命令执行依赖 npm 包 `@wetspace/wetspec-cli`（`npm install @wetspace/wetspec-cli` 或 `pnpm add @wetspace/wetspec-cli`）。Skill 不含 `scripts/`。

**CLI Guard**：每次工作流入口先检测 CLI；未安装则 HARD STOP 并提示用户安装，不得继续（见 `SKILL.md` 前置条件）。

**Py Guard（DP-0a）**：CLI 通过后 `wetspec py-install --check`；未就绪则 **AskQuestion**（安装 / Node 回退 / 暂停），禁止擅自 `py-install`（见 `decision_points.md` DP-0a）。

## 代码路径约定

- **Spec**：`specs/<中文模块>/` — 可与 PRD 一致用中文
- **源码**：`src/<module-slug>/` — **必须英文 kebab-case**；`design.md` 写映射，如 `用户登录` → `user-login`

## Change 目录

默认根路径：`wetspec/changes/<name>/`（可在 `specs/.wetspec.yaml` 设置 `changes_root`）。

**默认** `changes_root: wetspec/changes`。若历史 change 在其它路径，在 `specs/.wetspec.yaml` 中显式配置直至迁移。

```
<changes_root>/<name>/
  .wetspec.yaml          # scope: change
  proposal.md            # wetspec 设计产物
  design.md
  tasks.md
  diff.json              # PRD 对比（可选）
  wetspec-delta/         # Spec delta（增量 PRD 时）
```

## 状态字段

| 字段 | 说明 |
|------|------|
| `active_change` | 当前 change 名称（kebab-case） |
| `changes_root` | change 根目录，默认 `wetspec/changes` |
| `design_doc` | 技术设计路径，通常 `<changes_root>/<name>/design.md` |
| `build_target` | 实现目标 Spec 相对路径 |

`openspec_change` 已废弃，读取时回退到 `active_change`。

## 命令

```bash
# 设计
/wetspec-design

# 实现（读 design_doc + tasks.md + Spec）
/wetspec-build

# 验收
/wetspec-verify
```

## 决策点

每个 AskQuestion 选项须带 `（说明）`，详见 `decision_points.md`。

**设计阶段硬停（DP-3）**：`/wetspec-design` 产出 `proposal.md` / `design.md` / `tasks.md` 后，**必须** AskQuestion 并等待用户确认，**不得**在同一会话内静默衔接 `/wetspec-build`。用户说「完成需求」不豁免此步。见 `decision_points.md` §禁止因端到端指令跳过。

0. 工作流入口 → CLI Guard → Py Guard（DP-0a，`--check` 失败则 AskQuestion）
1. 首次 `wetspec init` → DP-0：单元测试框架（detect + configure；已配置则跳过）
2. Spec 完成 → DP-1：设计 / 归档 / 实现 / review / 暂停
3. 设计产物生成 → DP-3：确认 / 先 archive / 修改 / 暂停
4. verify 失败 → DP-4：修复 / 改 Spec / 接受偏差
5. archive dry-run → DP-5：确认回写 / 取消
