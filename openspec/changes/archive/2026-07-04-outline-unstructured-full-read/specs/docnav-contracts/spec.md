本 delta 定义 Docnav 共享契约中由 navigation 标准 `outline_mode` 触发的非结构化 outline 直接全文读取例外路径。

## ADDED Requirements

### Requirement: 生效策略触发的 outline 可以直接返回全文内容
Docnav 共享契约 MUST 保留普通文档的 `outline -> ref -> read` 主流程，同时 MUST 允许 navigation 标准 `outline_mode` 显式触发的非结构化文档在 `outline` 时直接返回全文内容。该非结构化结果 MUST 是可观察的 outline success shape，并 MUST 清楚标记其来源为 `path_rule` 或 `cost_threshold` 触发的自动全文读取。

#### Scenario: Agent 第一次 outline 即获得非结构化全文
- **WHEN** agent 对 `outline_mode` 为 `unstructured_full` 的文档执行 `outline`
- **THEN** 调用结果直接包含全文 content
- **THEN** 调用结果不要求 agent 再提交 ref 给 read
- **THEN** 调用结果不包含 page 或 continuation

#### Scenario: 普通文档仍使用 ref 流程
- **WHEN** agent 对 `outline_mode` 为 `structured` 的文档执行 `outline`
- **THEN** 调用结果包含 `kind: "structured"` 和 outline entries
- **THEN** agent 继续将 entry ref 原样传入 read

### Requirement: 非结构化全文结果不参与分页契约
标准 `outline_mode = "unstructured_full"` 触发的非结构化 outline 全文结果 MUST 不使用 page 表达读取位置，MUST 不裁剪非结构化全文结果，也 MUST 不产生下一页。Cost threshold is only a selector; it MUST NOT become an output limit once non-structured full-read is selected.

#### Scenario: 非结构化结果完整返回
- **WHEN** 标准 `outline_mode` 触发非结构化全文 outline
- **THEN** 成功结果仍包含完整原文内容
- **THEN** 成功结果不返回下一页 page
- **THEN** 文档说明该行为来自非结构化全文读取策略

#### Scenario: cost threshold 不是内容上限
- **WHEN** 标准 `outline_mode` 由 cost threshold 触发非结构化全文 outline
- **THEN** 成功结果仍包含完整原文内容
- **THEN** threshold 不裁剪 content
- **THEN** 成功结果不返回下一页 page
