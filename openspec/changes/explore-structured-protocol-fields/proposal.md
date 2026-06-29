本 change 探索 Docnav raw protocol 字段结构化方向；当前只在 `openspec/changes/explore-structured-protocol-fields/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## Why

当前协议中部分字段把机器可理解信息压缩进字符串，后续 cost、limit 和 display 相关变更需要先确认 raw protocol 与 readable 输出的分层边界。

## What Changes

- 审计 protocol request/result 中适合结构化的字段。
- 探索 `limit`、`cost`、outline entry、find match、info result、page 和 error details 的 owner 与候选 shape。
- 区分 raw protocol 的机器字段和 readable 输出的聚合展示。
- 输出候选协议方向、兼容策略和后续拆分建议。
- 非目标：本 change 不直接实现协议迁移，不提前固定所有字段最终 shape。

## Capabilities

### New Capabilities

- 无。

### Modified Capabilities

- `docnav-contracts`: 明确 raw protocol 优先结构化字段，readable 输出可重新聚合和组织。

## Impact

- 影响后续 protocol schema、examples、core/SDK/adapter request/result 类型和 readable 映射设计。
- 影响 `configure-pagination-defaults` 与 `use-token-based-document-cost` 的最终字段落点。
- 本 change 先产出探索结论和任务切分，不修改主规范、schema、examples 或代码。
