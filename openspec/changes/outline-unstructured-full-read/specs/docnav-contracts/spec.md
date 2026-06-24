本 delta 定义 Docnav 共享契约中非结构化 outline 直接全文读取的例外路径；它只在 `openspec/changes/outline-unstructured-full-read/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## ADDED Requirements

### Requirement: 配置触发的 outline 可以直接返回全文内容
Docnav 共享契约 MUST 保留普通文档的 `outline -> ref -> read` 主流程，同时 MUST 允许显式配置触发的非结构化文档在 `outline` 时直接返回全文内容。该非结构化结果 MUST 是可观察的 outline success shape，并 MUST 清楚标记其来源为配置触发的自动全文读取。

#### Scenario: Agent 第一次 outline 即获得非结构化全文
- **AND** 目标 path 命中非结构化文档配置
- **THEN** 调用结果直接包含全文 content
- **THEN** 调用结果不要求 agent 再提交 ref 给 read
- **THEN** 调用结果不包含 page 或 continuation

#### Scenario: 普通文档仍使用 ref 流程
- **AND** 目标 path 未命中非结构化文档配置
- **THEN** 调用结果仍返回 outline entries
- **THEN** agent 继续将 entry ref 原样传入 read

### Requirement: 非结构化全文结果不参与分页契约
配置触发的非结构化 outline 全文结果 MUST 不使用 page 表达读取位置。`limit_chars` 和 page 参数 MAY 仍由 CLI 或 protocol 参数层解析以保持输入兼容，但 MUST NOT 裁剪非结构化全文结果或产生下一页。

#### Scenario: 非结构化结果忽略字符预算分页
- **WHEN** 目标 path 命中非结构化文档配置
- **AND** 调用方传入 `--limit-chars` 或 page 参数
- **THEN** 成功结果仍包含完整原文内容
- **THEN** 成功结果不返回下一页 page
- **THEN** 文档说明该行为来自非结构化全文读取策略
