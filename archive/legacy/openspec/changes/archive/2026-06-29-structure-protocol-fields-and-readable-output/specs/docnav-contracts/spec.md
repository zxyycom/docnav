本 delta spec 定义 structured protocol fields 与 readable output organization 的目标契约。

## ADDED Requirements

### Requirement: Raw protocol exposes structured facts and readable output organizes them
Docnav raw protocol MUST 将机器可读事实表达为结构化 JSON 字段。Readable output MUST 把这些事实组织成面向阅读的 display text、summary 和布局，并且不成为 raw protocol fact source。

#### Scenario: 请求预算使用 canonical limit
- **WHEN** document operation request 携带分页预算
- **THEN** canonical protocol argument 是 `limit`
- **THEN** `limit` 是 positive integer，预算单位解释仍归 adapter 所有
- **THEN** schema、examples、typed arguments 和 operation handling 使用 `limit`

#### Scenario: cost 在协议中结构化，在 readable 输出中摘要化
- **WHEN** operation result 报告 cost
- **THEN** raw protocol 携带结构化 `cost.measurements[]`
- **THEN** 每个 measurement 包含机器可读的 `unit` 和 `value`
- **THEN** readable output 可以从这些 measurements 派生成紧凑成本摘要

#### Scenario: 导航条目分离 ref、事实字段和 display
- **WHEN** outline entries 或 find matches 返回
- **THEN** 每个 raw protocol item 保留 `ref` 作为 adapter-owned opaque string
- **THEN** label、location、summary、excerpt、rank、cost 和 metadata 等 item facts 在可用时使用结构化字段
- **THEN** readable output 拥有最终 display row

#### Scenario: info result 将 metadata 与摘要分离
- **WHEN** info 返回 document 或 adapter facts
- **THEN** raw protocol 使用结构化 document 和 adapter metadata 字段
- **THEN** readable output 可以把这些字段呈现为紧凑摘要

#### Scenario: error 和 warning 投影保留结构化 details
- **WHEN** protocol output 返回 `ok: false`
- **THEN** `error.code` 选择已文档化的结构化 `error.details` shape
- **THEN** `error.message` 和 `error.guidance` 保持展示字段
- **AND WHEN** readable output 包含 warnings
- **THEN** 每个 warning 包含稳定 `id`、`effect` 和 per-id 结构化 `details`
- **THEN** warning `reason` 保持展示文本

#### Scenario: continuation owner 保持稳定
- **WHEN** 引入 structured fields
- **THEN** `page` 仍是 protocol-owned next-page integer or null
