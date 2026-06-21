本 delta 定义 core CLI 对非结构化文档 outline 直接全文读取的配置和输出行为；它只在 `openspec/changes/outline-unstructured-full-read/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## ADDED Requirements

### Requirement: Core CLI 支持非结构化 outline 配置
`docnav` core CLI MUST 支持由生效配置声明的非结构化 outline 策略。命中该配置时，`docnav outline <path>` MUST 直接读取目标文档原文全文并返回非结构化 outline result；该路径 MUST NOT 生成、返回或要求调用方使用 ref。本 change MUST NOT 定义具体配置文件、格式或合并方式。

#### Scenario: 配置命中后 outline 自动全文读取
- **WHEN** 调用方执行 `docnav outline docs/raw-note.md`
- **AND** 生效配置声明 `docs/raw-note.md` 为非结构化文档
- **THEN** `docnav` 返回非结构化 outline readable/protocol 结果
- **THEN** 结果包含全文 content 和 content_type
- **THEN** 结果不包含 entries、ref、page 或 continuation
- **THEN** readable 输出说明该文本为配置触发的非结构化自动全文读取

#### Scenario: 未命中配置时保持普通 outline
- **WHEN** 调用方执行 `docnav outline docs/guide.md`
- **AND** 生效配置未声明该 path 为非结构化文档
- **THEN** `docnav` 按既有 adapter routing 和 invoke 流程执行结构化 outline
- **THEN** 输出包含 entries 和 page

#### Scenario: 输出说明非结构化策略命中
- **WHEN** 调用方执行 `docnav outline docs/raw-note.md`
- **AND** 该 path 命中非结构化 outline 配置
- **THEN** readable 输出包含稳定说明，表明该文本按配置作为非结构化文档自动全文读取
- **THEN** protocol/readable payload 包含等价的稳定原因字段
