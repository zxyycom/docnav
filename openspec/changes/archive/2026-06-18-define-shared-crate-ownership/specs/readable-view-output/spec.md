本 spec delta 定义 `docnav-output`、`docnav-readable` 和 `docnav-json-io` 的上下层关系，确保 document output orchestration 上移但 readable renderer 契约保持稳定。

## ADDED Requirements

### Requirement: Document output 编排必须位于 readable rendering 之上

`docnav-output` MUST 拥有 `readable-view`、`readable-json` 和 `protocol-json` 的 document operation output orchestration。`docnav-output` MUST 通过 document-only facade 接收调用方已构造的 operation、request id、output mode、document outcome 和 warnings。`docnav-readable` MUST 继续作为下层 owner，负责 readable payload/value helper、`ReadableViewKind`、renderer config、readable-view block rendering 和 conformance vectors。`docnav-json-io` MUST 作为更下层 helper 负责低层 JSON 写出。`docnav-output` MUST 调用 `docnav-readable` 完成 readable rendering，readable-view block framing 仍由 `docnav-readable` 拥有。

#### Scenario: readable-json 和 readable-view 共享同一个 readable payload

- **WHEN** document operation result 分别渲染为 `readable-json` 和 `readable-view`
- **THEN** `docnav-output` 构造或接收一个包含 operation fields、readable error fields 和 optional warnings 的完整 readable payload
- **THEN** `readable-json` 将该 readable payload 序列化为 JSON
- **THEN** `readable-view` 将同一个 readable value 和 `ReadableViewKind` 传给 `docnav-readable`

#### Scenario: protocol-json 保持 protocol-shaped

- **WHEN** document operation result 渲染为 `protocol-json`
- **THEN** `docnav-output` 向 stdout 写出 protocol response envelope
- **THEN** stdout 对该模式只包含一个 JSON value
- **THEN** 低层 JSON serialization 和 newline writing 可以通过 `docnav-json-io` 完成
- **THEN** warning metadata 不注入 protocol envelope
- **THEN** 需要表达 compatible CLI warnings 时，将其渲染为 stderr diagnostics

#### Scenario: readable renderer contract 仍在 docnav-readable

- **WHEN** readable-view rendering 执行 block pointer replacement、byte length calculation、marker framing 或 conformance vector validation
- **THEN** 该行为继续通过 `docnav-readable` 实现和测试
- **THEN** `docnav-output` 只选择 readable kind、在渲染前注入 warnings，并处理 output channel writing

#### Scenario: 非文档输出保持 owner-specific

- **WHEN** `docnav` 或 adapter 输出 help、version、manifest 或 probe
- **THEN** 该输出不成为 document output mode
- **THEN** 该输出不需要 readable-view framing
- **THEN** 该输出不通过 `docnav-output` 编排
- **THEN** 只有在不改变既有 schema、plain text 或 stderr boundary 时，才可以复用 `docnav-json-io` 或 diagnostics helper
