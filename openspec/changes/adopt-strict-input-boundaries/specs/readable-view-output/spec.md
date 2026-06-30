本 spec delta 定义 `adopt-strict-input-boundaries` 对 `readable-view-output` 的目标变更：readable 输出在成功时只承载成功 payload，在失败时承载 primary `DiagnosticRecord` 投影。

## MODIFIED Requirements

### Requirement: readable-view 必须成为统一默认阅读输出
`docnav` 和 adapter direct CLI 的 document operation MUST 支持 `readable-view`，并在 caller omits `--output` 时使用该模式。Outline、read、find、info、later composition operations 和 readable errors MUST 使用同一个 readable-view format 和 common renderer。

Invalid caller input MUST 按 owning output mode 渲染为 readable error。Structured readable error payloads MUST 写入 selected output mode 拥有的 structured output channel。

#### Scenario: 默认 outline 使用 readable-view
- **WHEN** 调用方执行 `docnav outline docs/guide.md` 且未传入 `--output`
- **THEN** stdout 从 pretty JSON header 开始
- **THEN** JSON header 保留 entries 和 page
- **THEN** stdout 不包含 protocol envelope

#### Scenario: 默认 read 使用 readable-view
- **WHEN** 调用方执行 `docnav read docs/guide.md --ref "<ref>"` 且未传入 `--output`
- **THEN** stdout 从 pretty JSON header 开始
- **THEN** JSON header 保留 ref、content 原位置、content_type、cost 和 page
- **THEN** content 字段通过显式 block 引用定位原文 block

#### Scenario: readable error 使用相同 view
- **WHEN** document operation 在 readable-view 模式下返回稳定错误
- **THEN** stdout 使用 readable-view pretty JSON header
- **THEN** JSON header 保留 code、error、details 和 guidance
- **THEN** stdout 只承载该 readable error payload 的 readable-view 表示
- **THEN** guidance 数组保持 header JSON 值，block 替换只应用于 renderer config 声明的字符串字段

#### Scenario: invalid caller input 使用 readable error
- **WHEN** caller input contains unknown argv, extra positional input or operation-inapplicable flags
- **THEN** readable-view output uses readable error projection
- **THEN** stdout contains the readable error payload

### Requirement: readable-view 和 readable-json 必须同源
实现 MUST 先构造完整的 typed readable payload，包含 operation readable fields 和 stable readable error fields。`readable-json` MUST 直接序列化该 payload。`readable-view` MUST 只对同一个 JSON value 应用 renderer-config block replacement 和 framing。两种输出 MUST 保留同一组 business fields 和 values。

Successful readable payloads MUST follow the owning operation success schema。Rejected argv、invalid config sources 和 automatic discovery all-failed lists 由 failure diagnostics 表达；后续成功的 discovery attempts 保持为 internal state。Future non-fatal operation notes MUST 由 owning operation/output contract 建模为 explicit business fields 或 guidance。

#### Scenario: success output 使用成功 payload shape
- **WHEN** 同一成功结果分别渲染为 readable-json 和 readable-view
- **THEN** readable-json 顶层字段符合 owning operation success schema
- **THEN** readable-view JSON header 字段符合 owning operation success schema
- **THEN** caller input diagnostics use readable error output

#### Scenario: invalid input 在两种阅读输出中都是错误
- **WHEN** 同一 document operation input 因严格输入校验失败
- **THEN** readable-json 输出 readable error payload
- **THEN** readable-view 输出相同 readable error payload 的 readable-view 表示
- **THEN** 两种输出都使用 readable error fields

#### Scenario: read 字段除 block 表示外一致
- **WHEN** 同一 read 结果分别渲染为 readable-json 和 readable-view
- **THEN** ref、content_type、cost 和 page 值一致
- **THEN** readable-view 的 `/content` block payload 等于 readable-json 的 content 字符串值

### Requirement: Document output 编排必须位于 readable rendering 之上
`docnav-output` MUST own document operation output orchestration for `readable-view`, `readable-json` and `protocol-json`。当 target projection 是 failure 时，`docnav-output` MUST 通过 document-only facade 接收 operation、request id、output mode、document outcome 和 primary `DiagnosticRecord`。`docnav-readable` MUST 保持 readable payload/value helpers、`ReadableViewKind`、renderer config、readable-view block rendering 和 conformance vectors 的 lower owner。`docnav-json-io` MUST 保持 low-level JSON writing 的 lower helper。

`docnav-output` MUST 将 rejected caller input 投影为 failure output。Strict input failure 在 protocol-json mode 下投影时，stdout MUST follow the protocol failure response contract。

#### Scenario: readable-json 和 readable-view 共享同一个 readable payload
- **WHEN** document operation result 分别渲染为 `readable-json` 和 `readable-view`
- **THEN** `docnav-output` 构造或接收一个包含 operation fields 或 readable error fields 的完整 readable payload
- **THEN** `readable-json` 将该 readable payload 序列化为 JSON
- **THEN** `readable-view` 将同一个 readable value 和 `ReadableViewKind` 传给 `docnav-readable`

#### Scenario: protocol-json 保持 protocol-shaped
- **WHEN** document operation result 渲染为 `protocol-json`
- **THEN** `docnav-output` 向 stdout 写出 protocol response envelope
- **THEN** stdout 对该模式只包含一个 JSON value
- **THEN** 低层 JSON serialization 和 newline writing 可以通过 `docnav-json-io` 完成
- **THEN** strict input failure 使用 protocol failure response contract

#### Scenario: readable renderer contract 仍在 docnav-readable
- **WHEN** readable-view rendering 执行 block pointer replacement、byte length calculation、marker framing 或 conformance vector validation
- **THEN** 该行为继续通过 `docnav-readable` 实现和测试
- **THEN** `docnav-output` 只选择 readable kind、在失败时映射 primary `DiagnosticRecord`，并处理 output channel writing

#### Scenario: 非文档输出保持 owner-specific
- **WHEN** `docnav` 或 adapter 输出 help、version、manifest 或 probe
- **THEN** 该输出使用 owner-specific output mode
- **THEN** 该输出使用 owner-specific framing
- **THEN** 该输出使用 owner-specific orchestration
- **THEN** 可复用 `docnav-json-io` 或 diagnostics helper 的前提是保持既有 schema、plain text 或 stderr boundary

### Requirement: renderer config 和 conformance vectors 必须支持跨语言消费
Readable-view renderer config 和 conformance vectors MUST 可供非 Rust 实现消费。Conformance 验证 MUST 聚焦语义字段：block pointer、byte length、block payload 还原、success header 字段语义、readable error projection 和空 block 合法。JSON header object key 顺序、block section 输出顺序和 Rust renderer 逐字节一致性不属于 conformance 判定依据。

#### Scenario: 跨语言 renderer 通过 conformance vectors 验证
- **WHEN** JavaScript renderer 实现根据本 spec 的 renderer config 渲染 readable-view
- **THEN** 跨语言 conformance 验证 block pointer 存在且正确
- **THEN** 跨语言 conformance 验证 `bytes` 等于该字段字符串 UTF-8 编码后的字节数
- **THEN** 跨语言 conformance 验证 block payload 与 readable-json 对应字段一致
- **THEN** 跨语言 conformance 按字段名和值判断 success header 或 readable error header 语义
- **THEN** 跨语言 conformance 不要求与 Rust renderer 逐字节一致

#### Scenario: 多 block 按 pointer 独立可定位
- **WHEN** renderer config 声明多个 block pointer 且跨语言 conformance 验证
- **THEN** 每个 block 的 pointer 和 byte length 与 header 对应 `$block` 引用一致
