# Verification

本文件记录本 change 在 2026-07-28 完成的最终验证证据。结果只证明当前工作树与 change artifact 已满足 apply 验收；change 保持未归档，归档仍需单独评估和执行。

## Authority and audit scope

本目录保存本次变更的提案、设计、迁移处置、任务和形成时验证证据，不是稳定规则或
当前实现状态的 owner：

- 日常测试变更从
  [`docs/testing/case-maintenance.md`](../../../../../../docs/testing/case-maintenance.md)
  读取当前账本规则。
- 当前测试实体的存在性与身份以当前源码、runner 报告和 project wrapper 为准。
- 当前 Case 内容以 `docs/testing/cases/<topic>.md` 为准，Topic 分类以
  `docs/testing/cases/topics.json` 为准。
- 本文件中的数量和验证结果是形成时观测；判断当前工作树时重新运行项目检查。

审计本 change 时依次读取 `proposal.md`、`design.md`、
`specs/test-evidence-management/spec.md`、`tasks.md` 和本文件。历史 Entry/Claim、
inventory/index 和旧 Case 只解释迁移背景，不参与当前查询或闭合，也不建立当前
Case、当前实体或产品测试义务。

## Focused verification

Status: passed

执行：

```bash
bun run test:test-evidence
bun run test:test-evidence-rules
```

- test-evidence focused suite：11 passed、0 failed，覆盖 Topic/Case parser、storage safety、scanner、coverage join、CLI contract 与 toolchain 边界。
- ast-grep scanner rule suite：9 passed、0 failed。
- 当前 owner/docs 契约由 workspace verification 与最终 `dnm outline` / legacy scan 共同核对；硬切换后不存在需要继续运行的旧 Evidence Claim docs integration suite。

## Full-tree Case closure

Status: passed

执行：

```bash
bun run test-evidence -- check --root .
```

最终集合为 537 个当前测试实体：393 Cargo、117 Bun、27 smoke。账本包含 117 个当前 Case、11 个 topics、546 条 Case/entity mappings；multi-mapped 计数为 9。static/runtime/entity mapping 与 Topic/Case 双向 coverage 全部闭合，diagnostics 为 0。

## Workspace verification

Status: passed with one non-blocking quality warning

执行：

```bash
bun run verify:docnav-workspace:required
openspec validate restore-semantic-test-case-ledger --type change --json --strict --no-interactive
bun run verify:docnav-workspace
```

- required profile：7 passed、1 quality warning、0 failed；warning 中没有 changed warning 或 regression，因此不阻断本 change。
- OpenSpec strict validation：passed，change valid 且 issues 为空。
- 完整 workspace profile：11 passed、1 quality warning、0 failed。该 warning 是已识别的质量债务，不阻断本 change。

## Migration disposition audit

Status: passed

历史基线共有 102 个 Case：101 implemented、1 planned。101 个 historical implemented Case 只作为逐项 review seed；最终 current ledger 保留 93 个语义连续的旧 ID，并新增 24 个由当前测试实体直接支持的语义 ID，合计 117 个 current Cases。只有本 change 实现开始前已有当前测试实体直接支持的历史语义才迁移；生产能力仍存在但缺起点直接实体，不会反向形成此 change 的产品测试义务。

- `WB-TYPED-FIELDS-PRESENCE-001`
- `WB-TYPED-FIELDS-METADATA-001`
- `WB-TYPED-FIELDS-CONSTRAINTS-001`
- `WB-TYPED-FIELDS-RANGES-001`

以上四个历史 ID 所述能力仍存在，但起点没有直接测试实体，因此未迁入 current ledger；本轮曾为迁移反向新增的产品测试和对应 current Case 已删除。后续是否补足直接测试由独立的 owner-driven product test change 评估，ID 不得换义复用。迁移统计把它们计入 8 个未进入 current ledger 的 historical implemented IDs，但其处置不是“生产能力已移除”。

`WB-TYPED-FIELDS-PROJECTION-001` 与 `WB-TYPED-FIELDS-COMPILE-001` 的处置原因不同：对应旧 FieldDefs derive/projection API 已移除，因此按生产能力移除规则退休，不是当前缺测试待办。

其余两个未进入 current ledger 的 historical implemented IDs 也有独立移除依据：`WB-PARAM-CLAP-001` 对应 consumer 已移除，`AUX-CASE-CATALOG-001` 对应旧 Case catalog 机制已移除。唯一 planned ID `BB-RELEASE-PACKAGE-001` 不迁入 current ledger，继续由规划 owner 承接。

最终审计没有发现空 Case、未迁移/退休 ID 复用或 committed 临时迁移映射。

## Scope and document inspection

Status: passed

`dnm outline`、局部 diff、路径过滤和 legacy terminology/dependency scan 已通过；当前 owner、skill、代码、测试与本 change 只保留目标 Topic/Case/test-entity 模型，允许的旧术语仅存在于 REMOVED context 与历史 artifact。最终验证没有归档本 change。
