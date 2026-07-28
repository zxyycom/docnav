# restore-semantic-test-case-ledger

本 change 把仓库测试证据维护切换为
`Topic -> Semantic Case(Owner, Proves, Entities) -> current test entity`，并让当前
测试实体与 Case 双向闭合。

## 状态与权威性

截至 `verification.md` 记录的 2026-07-28 验收，`tasks.md` 为 31/31 完成；change
仍未归档。本目录保存本次变更的提案、设计、迁移处置、任务和形成时验证证据，不是
稳定规则或当前实现状态的 owner：

- 日常测试变更从
  [`docs/testing/case-maintenance.md`](../../../docs/testing/case-maintenance.md)
  读取当前账本规则。
- 当前测试实体的存在性与身份以当前源码、runner 报告和 project wrapper 为准。
- 当前 Case 内容以 `docs/testing/cases/<topic>.md` 为准，Topic 分类以
  `docs/testing/cases/topics.json` 为准。
- 本目录中的数量和验证结果是形成时观测；判断当前工作树时重新运行项目检查。

## 审计读取顺序

1. `proposal.md`：恢复账本的原因、范围和受影响 surface。
2. `design.md`：本 change 的决策、取舍和历史 Case 迁移门槛。
3. `specs/test-evidence-management/spec.md`：相对前序 capability 的 contract delta。
4. `tasks.md`：实施与验收任务；`verification.md`：任务完成时取得的证据和迁移统计。

历史 Entry/Claim、inventory/index 和旧 Case 只解释迁移背景。它们不参与当前查询或
闭合，也不建立当前 Case、当前实体或产品测试义务。
