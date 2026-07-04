本 delta 定义 core CLI 对标准 `outline_mode = "unstructured_full"` 生效时，outline 直接全文读取的配置和输出行为。

## ADDED Requirements

### Requirement: Core CLI 支持非结构化 outline 标准调用参数
`docnav` core CLI MUST 支持由 navigation input resolution 产出的非结构化 outline 策略。标准 `outline_mode` 的生效值为 `unstructured_full` 时，`docnav outline <path>` MUST 在 selected adapter 正常 outline handler 之前返回非结构化 outline result；该路径 MUST NOT 生成、返回或要求调用方使用 ref。This change MUST NOT add a public CLI outline-mode override flag; normal and test coverage SHOULD exercise the behavior through config selectors.

`outline.mode_rules[]` 和 adapter-scoped cost threshold selector 的 source shape、priority 与 validation 语义由 navigation input resolution 拥有。

#### Scenario: 标准 outline_mode 触发 outline 自动全文读取
- **WHEN** 调用方执行 `docnav outline docs/raw-note.md`
- **AND** navigation 解析出的标准 `outline_mode` 为 `unstructured_full`
- **THEN** `docnav` 返回非结构化 outline readable/protocol 结果
- **THEN** 结果包含 `kind: "unstructured"`、全文 content 和 content_type
- **THEN** 结果不包含 entries、ref、page 或 continuation
- **THEN** readable 输出说明该文本为自动全文读取，并通过稳定 reason 区分 `path_rule` 或 `cost_threshold`

#### Scenario: 默认 structured 时保持普通 outline
- **WHEN** 调用方执行 `docnav outline docs/guide.md`
- **AND** navigation 解析出的标准 `outline_mode` 为默认值 `structured`
- **THEN** `docnav` 从当前 core release static adapter registry 选择 adapter implementation，并通过 linked adapter library/handler dispatch 执行结构化 outline
- **THEN** 输出包含 `kind: "structured"`、entries 和 page
- **THEN** entries 和 page 的结构化导航语义保持不变

#### Scenario: 输出说明非结构化策略命中
- **WHEN** 调用方执行 `docnav outline docs/raw-note.md`
- **AND** navigation 解析出的标准 `outline_mode` 为 `unstructured_full`
- **THEN** readable 输出包含稳定说明，表明该文本作为非结构化文档自动全文读取
- **THEN** protocol/readable payload 包含等价的稳定原因字段
