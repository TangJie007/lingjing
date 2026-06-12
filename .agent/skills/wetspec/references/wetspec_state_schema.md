# wetspec 状态文件 Schema

> wetspec 内置脚本化状态管理（设计时参考了 Comet 思路，**运行时无 Comet 依赖**）。
> 双层状态：
> - 主库：`specs/.wetspec.yaml`（`scope: main`）
> - Change：`<changes_root>/<name>/.wetspec.yaml`（`scope: change`，默认 `wetspec/changes/`）

## 文件位置

```
specs/.wetspec.yaml                        # 主库状态
<changes_root>/<name>/.wetspec.yaml         # change 状态（默认 wetspec/changes/）
<changes_root>/<name>/wetspec-delta/       # delta Spec（仅 affected_specs）
<changes_root>/<name>/design.md            # wetspec 技术设计（/wetspec-design）
```

## 字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| `version` | string | 状态 Schema 版本，主库 `1.0` / change `1.1` |
| `scope` | string | `main`（主库）或 `change`（变更隔离） |
| `workflow` | string | 主库 `full`；change `delta` |
| `change_name` | string | change 名称（仅 scope=change） |
| `main_specs` | string | 主库相对路径（仅 scope=change），如 `specs/` |
| `delta_dir` | string | delta 目录名，默认 `wetspec-delta` |
| `phase` | string | 当前阶段：`idle` `parse` `awaiting-unit-test` `update` `sync` `specs-ready` `design` `build` `verify` `archive` `done` |
| `mode` | string | 同步模式：`incremental`（默认）或 `full` |
| `prd.current` | string | 当前 PRD 文件名 |
| `prd.previous` | string | 上一版 PRD 文件名 |
| `last_diff` | string | 最近一次 diff.json 路径 |
| `last_sync_at` | string | ISO 时间戳 |
| `affected_specs` | array | 最近一次变更影响的 Spec 相对路径 |
| `archived` | boolean | 是否已归档 |
| `auto_transition` | boolean | 是否在用户确认后自动进入设计阶段，默认 `true` |
| `changes_root` | string | change 根目录，默认 `wetspec/changes` |
| `active_change` | string | 当前 change 名称（设计 / 增量 PRD） |
| `openspec_change` | string | **已废弃**，与 `active_change` 同步，兼容旧文件 |
| `design_doc` | string | 技术设计路径，通常 `<changes_root>/<name>/design.md` |
| `build_target` | string | 当前实现的 Spec 相对路径 |
| `verify_result` | string | 验收结果：`pending` `pass` `fail` |
| `verification_report` | string \| null | **已废弃**，保留兼容；验收结果写入各 Spec YAML |
| `unit_test` | object \| null | 单元测试基建（DP-0 配置）；见下表 |
| `unit_test.framework` | string \| null | `node:test` `vitest` `jest` `pytest` |
| `unit_test.deferred` | boolean | 是否暂缓选定（`true` 时 build 前须再配置） |
| `unit_test.path` | string | 单元测试文件 glob，如 `src/**/__tests__/**/*.test.js` |
| `unit_test.command` | string | build 后执行的命令，如 `npm run test:unit` |
| `unit_test.configured_at` | string | 配置日期 ISO `YYYY-MM-DD` |
| `unit_test.ac_runner` | string | AC 层 runner 说明，固定 `node:test`（Node 项目） |

**单层测试**（实现 + 验收）：

| 路径 | build | verify |
|------|-------|--------|
| `src/<module-slug>/__tests__/`（或 `unit_test.path`） | `unit_test.command` | `wetspec verify` 按 LOG/AC describe 嵌套逐条跑 |

## 阶段转换（Guard）

由 `wetspec_state.js check --transition <name>` 强制执行：

| Transition | 前置 phase | 目标 phase | 前置条件 |
|------------|-----------|-----------|----------|
| `start-parse` | idle, done | parse | — |
| `await-unit-test` | parse | awaiting-unit-test | 用户选定 vitest/jest/pytest 且须自装依赖 |
| `unit-test-ready` | awaiting-unit-test | parse | `unit-test check` 通过且已 configure |
| `parse-complete` | parse | specs-ready | — |
| `start-update` | idle, done | update | prd.current |
| `update-complete` | update | specs-ready | last_diff |
| `start-sync` | idle, update, done | sync | prd.current |
| `sync-complete` | sync | specs-ready | — |
| `start-design` | specs-ready, done | design | 用户确认进入设计 |
| `skip-design` | specs-ready | done | 用户选择暂不设计 |
| `design-complete` | design | done | design_doc |
| `start-build` | done, verify | build | — |
| `build-complete` | build | verify | build_target |
| `verify-pass` | verify | done | verify_result |
| `verify-fail` | verify | build | — |

失败时输出 `[HARD STOP]`，Agent 不得跳过。

## 附录：与 Comet 的参考对照（非依赖）

wetspec **不要求**安装 Comet。下表仅说明 wetspec 自研脚本与 Comet 概念的对应关系，便于从 Comet 迁移或对照阅读：

| Comet（参考） | wetspec（内置） |
|---------------|-----------------|
| `.comet.yaml` | `.wetspec.yaml` |
| `comet-state.sh` | `wetspec_state.js` |
| `comet-guard.sh` | `wetspec_state.js check --transition` |
| `comet-design` | `/wetspec-design` |
| `comet-build` | `/wetspec-build` |
| `comet-verify` | `/wetspec-verify` |
| `comet-archive`（delta→main） | `wetspec_archive.js` |
| `comet doctor` | `wetspec_doctor.js` |

## Change 隔离（scope=change）

| 脚本 | 作用 |
|------|------|
| `wetspec_preflight.js` | 多人协作预检 |
| `wetspec_change.js` | init / set-manifest / validate-delta |
| `wetspec_archive.js` | delta → 主 specs 回写 |

详见 `change_isolation.md`。

## wetspec 独有优势

- **Change 隔离**：delta 只含 affected_specs，archive 回写主库，避免多人全量复制
- **增量更新**：`compare_prd.js --spec-dir` 映射 `affected_specs`，只改受影响 YAML
- **PRD 追溯**：`check_coverage.js` 验证 PRD 功能是否都有 Spec
- **双格式 Spec**：标准 metadata 格式 + legacy spec 块兼容
