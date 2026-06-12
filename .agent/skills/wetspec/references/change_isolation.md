# Change 隔离：delta → main archive

> 多人协作下，wetspec 在 **change 工作区**（默认 `wetspec/changes/<name>/`）内只存 delta，archive 时回写主 `specs/`。不要求 OpenSpec / Comet。

## 目录结构

```
specs/                                    # 主库（canonical，每功能一份）
  .wetspec.yaml                           # 主库状态 scope=main
  用户登录/
    手机号+验证码登录_spec.yaml

<changes_root>/<change-name>/             # 变更隔离区（默认 wetspec/changes/）
  .wetspec.yaml                           # change 状态 scope=change
  proposal.md / design.md / tasks.md
  wetspec-delta/                          # ⚠️ 仅 affected_specs
    MANIFEST.json
    用户登录/
      手机号+验证码登录_spec.yaml          # 仅此一个，非全量 5 个功能
```

## 禁止 vs 允许

| 操作 | 允许 | 禁止 |
|------|------|------|
| 在 change 内更新 LOG-001 | ✅ `wetspec-delta/用户登录/手机号+验证码登录_spec.yaml` | ❌ |
| 复制整个 `specs/用户登录/` 到 change | | ❌ 全量复制 |
| 直接改主 `specs/`（有活跃 change 时） | | ❌ 应走 change + archive |
| archive 后回写主库 | ✅ 只复制 affected_specs | |

## 工作流命令

```bash
# 1. 预检（多人协作）
wetspec preflight wetspec/changes/<name> --main-specs specs/ --prd PRD_new.md

# 2. 初始化 change 工作区
wetspec change init wetspec/changes/<name> --main-specs specs/ --prd PRD_new.md

# 3. PRD diff → 更新 MANIFEST
wetspec compare old.md new.md --output diff.json --spec-dir specs/
wetspec change set-manifest wetspec/changes/<name> --diff diff.json

# 4. AI 只写入 wetspec-delta/<affected_specs>

# 5. 校验 delta 合规
wetspec change validate-delta wetspec/changes/<name>

# 6. archive 回写主 specs
wetspec archive wetspec/changes/<name> --dry-run
wetspec archive wetspec/changes/<name>
```

## 冲突处理

1. 不同 change 改**不同功能** → 互不影响，各自 archive
2. 两个 change 改**同一功能** → Git PR 冲突，需人工合并或以最新 PRD 重跑 compare_prd
3. archive 冲突后 → 以 PRD 为真相源，对主库重跑增量更新，**不要手 merge changelog**

## MANIFEST.json

```json
{
  "change_name": "prd-user-login-v11",
  "main_specs": "specs/",
  "affected_specs": [
    { "path": "用户登录/手机号+验证码登录_spec.yaml", "change_type": "modified" }
  ],
  "rules": "本目录仅允许存放 affected_specs 中的 Spec YAML"
}
```

`validate-delta` 会拒绝 MANIFEST 以外的 YAML 文件。
