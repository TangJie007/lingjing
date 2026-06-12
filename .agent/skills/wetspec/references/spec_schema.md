# PRD-to-Spec YAML Schema 参考

> 本文档详细说明 Spec YAML 文件中每个字段的含义、类型和约束。

## 顶层结构

```yaml
metadata:       # 元数据（必填）
description:    # 功能描述（必填）
user_stories:   # 用户故事列表（必填，至少1条）
acceptance_criteria:  # 验收标准列表（必填，至少1条）
api_spec:       # API 接口定义（可选）
data_model:     # 数据模型（可选）
dependencies:   # 依赖关系（可选）
non_functional: # 非功能性需求（可选）
ui_requirements:# 界面需求（可选）
changelog:      # 变更历史（必填，至少1条）
```

## metadata 字段

| 字段 | 类型 | 必填 | 说明 | 约束 |
|------|------|------|------|------|
| `id` | string | ✅ | 功能唯一标识 | 建议格式：`{模块缩写}-{序号}`，如 `USR-001` |
| `module` | string | ✅ | 所属模块名（中文） | 如"用户管理" |
| `feature` | string | ✅ | 功能名称（中文） | 如"用户注册" |
| `version` | string | ✅ | 语义化版本号 | 格式 `X.Y.Z`，默认 `1.0.0` |
| `status` | string | ✅ | 当前状态 | 枚举：`draft` `review` `approved` `implemented` `deprecated` |
| `priority` | string | ✅ | 优先级 | 枚举：`P0` `P1` `P2` `P3` |
| `created_at` | string | ✅ | 创建日期 | 格式 `YYYY-MM-DD` |
| `updated_at` | string | ✅ | 最后更新日期 | 格式 `YYYY-MM-DD` |
| `author` | string | ⬜ | 作者/负责人 | |
| `prd_source` | string | ⬜ | 来源 PRD 文件名 | 如 `PRD_v2.0.md` |

## description 字段

| 字段 | 类型 | 说明 |
|------|------|------|
| `summary` | string | 一句话描述，不超过140字 |
| `detail` | string | 详细描述，支持多行文本 |
| `scope` | string | 功能范围说明，明确边界 |

## user_stories 条目

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | string | 用户故事编号，如 `US-001` |
| `role` | string | 角色，如"管理员""普通用户" |
| `action` | string | 期望执行的操作 |
| `goal` | string | 期望达成的目标 |
| `priority` | string | 枚举：`P0` `P1` `P2` `P3` |

## acceptance_criteria 条目

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | string | 验收标准编号，如 `AC-001` |
| `description` | string | 验收标准描述 |
| `expected_result` | string | 预期结果 |
| `test_method` | string | 测试方式：`manual` `auto` `both` |

## changelog 条目（增量更新核心）

每次更新 Spec 时**必须追加**一条 changelog：

| 字段 | 类型 | 说明 |
|------|------|------|
| `version` | string | 变更后的版本号 |
| `date` | string | 变更日期 |
| `changes` | list | 变更明细列表 |

每个 change 明细：

| 字段 | 类型 | 说明 |
|------|------|------|
| `type` | string | 变更类型：`added` `modified` `removed` `deprecated` |
| `field` | string | 变更的字段路径，如 `user_stories[0].action` |
| `old_value` | string | 旧值（modified/removed 时填写） |
| `new_value` | string | 新值（added/modified 时填写） |
| `reason` | string | 变更原因 |
| `related_prd_change` | string | 关联的 PRD 变更描述 |

## 增量更新工作流

```
新版 PRD 到达
    │
    ▼
compare_prd.py 对比新旧 PRD
    │
    ▼
输出 diff.json（结构化差异报告）
    │
    ▼
AI 分析 diff，识别受影响的模块/功能
    │
    ▼
只读取并更新受影响的 Spec 文件（不碰其它文件）
    │
    ▼
更新 changelog，保存
    │
    ▼
validate_spec.py 校验更新后的文件
```

## 状态流转图

```
draft ──→ review ──→ approved ──→ implemented
  │                                    │
  └────────────── deprecated ←─────────┘
```

## ID 编号建议

| 模块 | ID 前缀 | 示例 |
|------|---------|------|
| 用户管理 | `USR` | `USR-001` |
| 订单管理 | `ORD` | `ORD-001` |
| 支付模块 | `PAY` | `PAY-001` |
| 商品管理 | `PRD` | `PRD-001` |
| 消息通知 | `MSG` | `MSG-001` |
