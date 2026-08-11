本 spec delta 定义 protocol、JSON IO 和 adapter SDK helper 的边界，确保 direct CLI 去重不削弱 invoke 严格校验或 adapter-owned 语义。

## ADDED Requirements

### Requirement: Protocol 和 adapter SDK helper 必须保持进程边界契约

`docnav-protocol`、`docnav-json-io` 和 `docnav-adapter-sdk` MUST 只在不破坏当前 protocol、direct CLI 和 adapter process boundary 的位置暴露共享 helper。Adapter `invoke` stdin JSON MUST 保持严格 protocol input；adapter direct CLI document command MAY 复用 direct CLI loose argv 和 document output helper。

#### Scenario: Protocol decode helper 保持严格 schema 和 semantic validation

- **WHEN** 共享代码 decode protocol request、protocol response、manifest 或 probe JSON value
- **THEN** decode pipeline 在把 JSON value 当作 typed contract data 前，先按 owning schema 校验
- **THEN** typed deserialization 和 semantic validation 在 schema validation 之后执行
- **THEN** 调用方 surface 保持既有 stable error category、field path、diagnostic text 和 exit behavior

#### Scenario: Adapter invoke 保持严格 protocol decoding

- **WHEN** adapter `invoke` 收到包含 unknown fields、missing required fields 或 wrong argument types 的 stdin JSON
- **THEN** SDK strict protocol decoding 按 invoke contract 拒绝该请求
- **THEN** 不应用 `docnav-cli-args` loose argv rule
- **THEN** failure 仍是 protocol-shaped failure response

#### Scenario: Adapter direct CLI document command 复用共享 helper

- **WHEN** adapter direct CLI document operation 成功或返回 stable error
- **THEN** SDK 可以使用共享 diagnostics 表达 warning envelope 和 stderr warning text
- **THEN** SDK 可以使用 `docnav-output` 执行 document output mode dispatch
- **THEN** manifest、probe 和 help output 保持既有 adapter contract 或 plain text boundary
- **THEN** manifest 和 probe 的 machine-readable JSON 可以复用 `docnav-json-io`
- **THEN** manifest、probe 和 help output 不通过 `docnav-output` 编排

#### Scenario: Adapter SDK paging helper 保持 format-neutral

- **WHEN** adapter 使用 SDK paging helper
- **THEN** helper 处理 character budget、text 或 entry pagination、next page calculation 和 truncation mechanics
- **THEN** helper 不生成 refs、不解析 refs、不检查 Markdown heading hierarchy，也不定义 adapter-specific display semantics

#### Scenario: Request id helper 只拥有格式不拥有 surface policy

- **WHEN** core、SDK 或 output code 需要 generated request id fallback
- **THEN** 它可以使用 `docnav-protocol` 提供的共享 request id helper
- **THEN** 调用方仍决定何时保留 incoming request id、何时使用 unknown placeholder，以及 request id 暴露在哪个 surface
