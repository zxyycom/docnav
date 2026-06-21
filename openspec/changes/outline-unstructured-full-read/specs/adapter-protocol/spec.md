本 delta 定义非结构化 outline 全文结果在共享协议类型中的形态；它只在 `openspec/changes/outline-unstructured-full-read/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## ADDED Requirements

### Requirement: OutlineResult 支持配置触发的非结构化全文形态
`docnav-protocol` MUST 将 outline success result 表达为可判别 union：普通结构化形态包含 `entries` 和 `page`；配置触发的非结构化全文形态包含全文 `content`、`content_type`、`cost` 和稳定原因字段，并 MUST NOT 包含 `entries`、`ref`、`page` 或 continuation 字段。

#### Scenario: 构造非结构化 outline 成功响应
- **WHEN** 调用方使用共享协议类型构造配置触发的非结构化 `outline` 成功响应
- **THEN** 响应包含 `protocol_version`、`request_id`、`operation: "outline"`、`ok: true` 和 outline result
- **THEN** result 可被识别为非结构化全文形态
- **THEN** result 包含全文 `content`、`content_type`、`cost` 和 `reason: "configured_unstructured_document"` 或等价稳定原因
- **THEN** result 不包含 `entries`、`ref`、`page` 或 continuation 字段

#### Scenario: 普通 outline 仍使用结构化 entries
- **WHEN** outline 未命中非结构化配置
- **THEN** 成功 result 仍使用结构化 entries 形态
- **THEN** 每条 entry 仍包含 adapter 生成的 `ref` 和 `display`
- **THEN** `page` 仍按普通 outline 分页规则表达是否可继续
