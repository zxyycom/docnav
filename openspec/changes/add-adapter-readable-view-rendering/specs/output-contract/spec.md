本 delta spec 定义 `readable-view` adapter 可选文本渲染 hook 与 core generic fallback 的输出契约；当前文档只在 `openspec/changes/add-adapter-readable-view-rendering/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## ADDED Requirements

### Requirement: adapter readable-view renderer hook 必须是可选 presentation capability
Selected adapter MAY provide a readable-view renderer hook for successful document operation results. The hook MUST receive the completed operation success payload and render context, and MUST return a single UTF-8 text value or `unsupported`. The hook MUST NOT read CLI argv、stdin、stdout、stderr、process cwd or process exit code, and MUST NOT alter adapter routing、operation success payload、ref、page、diagnostic or machine output semantics.

#### Scenario: adapter renderer returns text for supported operation
- **WHEN** a selected adapter supports readable-view rendering for `find`
- **AND** the operation success payload has already been produced
- **THEN** the output layer passes that success payload and render context to the adapter renderer
- **THEN** the adapter renderer returns a UTF-8 text value
- **THEN** stdout contains that text through the `readable-view` output mode
- **THEN** `readable-json` and `protocol-json` for the same operation remain based on the original success payload

#### Scenario: adapter renderer is not user configurable
- **WHEN** user config、project config、environment variables、CLI flags or linked adapter descriptor metadata are evaluated
- **THEN** none of those surfaces selects or injects a readable-view renderer implementation
- **THEN** renderer availability is determined by the selected adapter capability known to the implementation

#### Scenario: adapter renderer cannot change operation facts
- **WHEN** an adapter-rendered readable-view uses a custom text presentation
- **THEN** that text belongs only to the readable-view success projection
- **THEN** complete ref、page、match、entry and result facts remain available through `readable-json` or `protocol-json`

## MODIFIED Requirements

### Requirement: readable-view 必须成为统一默认阅读输出
`docnav` document operations MUST 支持 `readable-view`，并在 caller omits `--output` 时使用该模式。Successful `readable-view` output MUST render as a single human-readable UTF-8 text stream selected by document output orchestration. The output layer MAY use a selected adapter's optional readable-view renderer for successful operation results; otherwise it MUST use the core generic readable-view fallback. Readable errors MUST use the readable error projection owned by the output layer and MUST NOT require adapter readable-view rendering.

Invalid caller input MUST 按 owning output mode 渲染为 readable error。Structured readable error payloads MUST 写入 selected output mode 拥有的 structured output channel.

#### Scenario: 默认 outline 使用 readable-view
- **WHEN** 调用方执行 `docnav outline docs/guide.md` 且未传入 `--output`
- **THEN** stdout uses the `readable-view` output mode
- **THEN** stdout is a human-readable text stream
- **THEN** stdout 不包含 protocol envelope
- **THEN** stdout is not required to start with a pretty JSON header when adapter rendering is used

#### Scenario: 默认 read 使用 fallback when adapter renderer is unavailable
- **WHEN** 调用方执行 `docnav read docs/guide.md --ref "<ref>"` 且未传入 `--output`
- **AND** the selected adapter does not provide a readable-view renderer for `read`
- **THEN** stdout is produced by the core generic readable-view fallback
- **THEN** stdout preserves the readable read facts required by the fallback renderer
- **THEN** stdout 不包含 protocol envelope

#### Scenario: readable error 使用 output-owned projection
- **WHEN** document operation 在 readable-view 模式下返回稳定错误
- **THEN** adapter readable-view renderer is not invoked for that failure
- **THEN** stdout or stderr follows the readable error projection for the primary diagnostic
- **THEN** the failure output only represents that readable error payload

#### Scenario: invalid caller input 使用 readable error
- **WHEN** caller input contains unknown argv, extra positional input or operation-inapplicable flags
- **THEN** readable-view output uses readable error projection
- **THEN** linked adapter readable-view renderer does not execute

### Requirement: renderer config 必须是仓库内契约
Core generic readable-view fallback MUST use repository-owned renderer configuration or repository-owned renderer code. Adapter readable-view renderer availability and behavior MUST be repository implementation contract for the adapter, not user configuration, project configuration, environment variables, CLI flags or linked adapter descriptor metadata. Generic fallback conformance MUST remain deterministic for the fallback format it owns. Adapter-rendered readable-view text semantics MUST be defined only by the selected adapter's own contract, not by the generic hook.

#### Scenario: fallback renderer config remains repository-owned
- **WHEN** core generic readable-view fallback uses a configured structured renderer
- **THEN** the config comes from repository-owned code or committed config
- **THEN** user config、project config、environment variables、CLI flags and linked adapter descriptor metadata do not change fallback block fields or fallback renderer behavior

#### Scenario: adapter renderer does not use renderer config blocks
- **WHEN** selected adapter readable-view renderer returns custom text
- **THEN** the output layer writes that text as the readable-view success projection
- **THEN** the adapter-rendered text is not required to preserve JSON header fields or block pointers
- **THEN** machine-readable fields remain available from `readable-json` and `protocol-json`

### Requirement: block 字段必须由 renderer config 显式声明
When core generic readable-view fallback uses structured block replacement, each fallback view kind MUST declare its block fields through repository-owned renderer config. The fallback renderer MUST only replace JSON Pointer fields listed by that config; undeclared fields MUST remain in the fallback's structured header. Adapter-rendered readable-view text MUST NOT be required to use block replacement, block pointers or structured JSON headers.

#### Scenario: fallback read content 由 config 外置
- **WHEN** core generic fallback readable read renderer config 声明 block pointer `/content`
- **THEN** fallback rendering 将 JSON header 的 content 值替换为 `$block` 引用
- **THEN** fallback rendering 把原 content 字符串写入 `/content` block

#### Scenario: fallback 未声明多行字符串保持 JSON 字符串
- **WHEN** fallback readable payload 中某个未被 renderer config 声明的字符串包含换行
- **THEN** 该字段仍作为合法 JSON 字符串保留在 fallback header 中
- **THEN** block section 只覆盖 fallback renderer config 声明的字段

#### Scenario: adapter-rendered text bypasses block replacement
- **WHEN** selected adapter readable-view renderer succeeds
- **THEN** output orchestration writes the returned text as the readable-view success projection
- **THEN** no block pointer lookup, block identity check or block payload restoration is required for that adapter-rendered text

### Requirement: readable-view 必须使用可定界格式
Readable-view renderer MUST complete rendering in memory before stdout writing. Adapter-rendered readable-view MUST produce one UTF-8 text value. The generic hook MUST NOT require that text to be original-format-like, lossy, omission-aware, ref-aware or continuation-aware; those semantics belong to adapter-specific renderer contracts when they exist. Core generic fallback MAY use a delimited structured text format such as pretty JSON header plus block sections; when it does, its delimiter, byte length and framing rules MUST remain deterministic for that fallback output.

#### Scenario: adapter-rendered text is a complete stdout payload
- **WHEN** adapter readable-view renderer succeeds
- **THEN** stdout contains the complete returned text as the readable-view success payload
- **THEN** the output layer does not append protocol envelope fields
- **THEN** adapter-specific markers or layout choices in that text belong only to the readable-view projection

#### Scenario: generic fallback can still use block references
- **WHEN** core generic fallback uses block replacement for `/content`
- **THEN** the fallback output preserves its configured block references and byte length rules
- **THEN** tests can validate fallback block pointer、byte length and payload restoration for fallback cases

#### Scenario: no partial stdout on render failure before fallback
- **WHEN** adapter renderer returns an error before stdout writing
- **THEN** stdout has not been written
- **THEN** output orchestration can use the generic fallback for the same success payload

### Requirement: readable-view 和 readable-json 必须同源
实现 MUST 先构造完整的 typed readable payload or protocol-backed success facts for the document operation. `readable-json` MUST directly serialize the readable payload it owns. `readable-view` MUST render from the same completed success facts, either through adapter text rendering or through the generic fallback. Adapter-rendered `readable-view` MAY use any custom text presentation allowed by that adapter's own contract, but MUST NOT become the source of truth for `readable-json` or `protocol-json`.

Successful readable payloads MUST follow the owning operation success schema. Rejected argv、invalid config sources 和 automatic discovery all-failed lists 由 failure diagnostics 表达；后续成功的 discovery attempts 保持为 internal state. Future non-fatal operation notes MUST 由 owning operation/output contract 建模为 explicit business fields 或 guidance.

#### Scenario: success output keeps machine facts stable
- **WHEN** 同一成功结果分别渲染为 readable-json、protocol-json 和 readable-view
- **THEN** readable-json 顶层字段符合 owning operation success schema
- **THEN** protocol-json uses the protocol response envelope and result shape
- **THEN** readable-view may use adapter-rendered text or generic fallback
- **THEN** readable-view text does not alter the machine-readable success facts

#### Scenario: invalid input 在两种阅读输出中都是错误
- **WHEN** 同一 document operation input 因严格输入校验失败
- **THEN** readable-json 输出 readable error payload
- **THEN** readable-view 输出同一 primary diagnostic 的 readable error projection
- **THEN** adapter success renderer is not invoked

#### Scenario: adapter text remains presentation only
- **WHEN** 同一 read 结果分别渲染为 readable-json 和 adapter-rendered readable-view
- **THEN** readable-json preserves the read success payload fields
- **THEN** readable-view MAY use adapter-defined custom text presentation
- **THEN** that text does not change the read success payload or page facts

### Requirement: renderer 失败必须有稳定边界
Readable-view renderer MUST 在写 stdout 前完成内存渲染。Adapter renderer absence, `unsupported`, or renderer-local failure MUST fall back to the core generic readable-view renderer for successful operation results. Generic fallback render failure MUST use a stable diagnostic, MUST avoid partial stdout, and MUST return the owning internal error exit code. Output I/O errors after completed rendering MUST continue to use the existing I/O failure path.

#### Scenario: adapter renderer unsupported falls back
- **WHEN** selected adapter returns `unsupported` for readable-view rendering
- **THEN** output orchestration uses the core generic fallback for the same success payload
- **THEN** the document operation does not fail solely because the adapter renderer is unsupported

#### Scenario: adapter renderer failure falls back
- **WHEN** selected adapter readable-view renderer fails before stdout writing
- **THEN** stdout remains empty before fallback
- **THEN** output orchestration uses the core generic fallback
- **THEN** the renderer-local failure does not change `readable-json` or `protocol-json`

#### Scenario: fallback failure remains stable
- **WHEN** generic fallback rendering fails after adapter renderer is unavailable or failed
- **THEN** stdout 为空
- **THEN** stderr contains the stable readable-view render failure diagnostic
- **THEN** 命令使用内部错误 exit code 退出

#### Scenario: stdout I/O 错误按既有 I/O 失败处理
- **WHEN** readable-view 已完成内存渲染但写 stdout 失败
- **THEN** CLI 按项目既有 I/O 错误路径退出
- **THEN** 该 I/O 失败使用项目既有 I/O 错误路径表达

### Requirement: renderer config 和 conformance vectors 必须支持跨语言消费
Core generic readable-view fallback renderer config and fallback conformance vectors MUST remain consumable by non-Rust implementations when that fallback uses structured block rendering. Generic adapter hook conformance MUST cover hook selection, unsupported/failure fallback, machine output bypass and absence of protocol envelope in successful readable-view text. Adapter-rendered text semantics beyond those generic boundaries MUST be validated by the selected adapter's own spec and tests. Cross-language fallback conformance MUST NOT require adapter-rendered text to preserve fallback JSON header or block section bytes.

#### Scenario: fallback renderer vectors validate block semantics
- **WHEN** JavaScript renderer 实现根据 generic fallback renderer config 渲染 fallback readable-view
- **THEN** 跨语言 conformance 验证 fallback block pointer 存在且正确
- **THEN** 跨语言 conformance 验证 `bytes` 等于该字段字符串 UTF-8 编码后的字节数
- **THEN** 跨语言 conformance 验证 block payload 与 fallback readable field 一致
- **THEN** 跨语言 conformance 不要求与 Rust fallback renderer 逐字节一致

#### Scenario: adapter renderer vectors validate adapter-owned text semantics
- **WHEN** an adapter readable-view renderer is validated through adapter-specific smoke or golden fixtures
- **THEN** validation checks only the generic hook boundaries plus the text semantics declared by that adapter's own spec
- **THEN** validation does not require JSON header object keys, block pointers or block section output for adapter-rendered text

### Requirement: Document output 编排必须位于 readable rendering 之上
`docnav-output` MUST own document operation output orchestration for `readable-view`, `readable-json` and `protocol-json`. 当 target projection 是 failure 时，`docnav-output` MUST 通过 document-only facade 接收 operation、request id、output mode、document outcome 和 primary `DiagnosticRecord`. For successful `readable-view`, `docnav-output` MUST choose between selected adapter readable-view renderer and core generic fallback before stdout writing. `docnav-readable` or an equivalent lower owner MUST keep generic fallback helpers and conformance materials. `docnav-json-io` MUST 保持 low-level JSON writing 的 lower helper.

`docnav-output` MUST 将 rejected caller input 投影为 failure output. Strict input failure 在 protocol-json mode 下投影时，stdout MUST follow the protocol failure response contract.

#### Scenario: readable-json and protocol-json bypass adapter presentation text
- **WHEN** document operation result 渲染为 `readable-json` or `protocol-json`
- **THEN** `docnav-output` serializes the owning JSON payload or protocol envelope
- **THEN** selected adapter readable-view renderer is not invoked
- **THEN** stdout 对 machine-readable mode 只包含一个 JSON value

#### Scenario: readable-view success chooses adapter renderer before fallback
- **WHEN** document operation result 渲染为 `readable-view`
- **AND** selected adapter supports readable-view rendering for that operation
- **THEN** `docnav-output` passes the completed success facts and render context to that renderer
- **THEN** successful renderer text is written as readable-view stdout

#### Scenario: readable-view fallback remains output-owned
- **WHEN** selected adapter has no readable-view renderer or returns `unsupported`
- **THEN** `docnav-output` uses the core generic fallback
- **THEN** fallback rendering and failure mapping remain owned by the output layer and lower readable helpers

#### Scenario: 非文档输出保持 owner-specific
- **WHEN** `docnav` 或 adapter 输出 help、version、manifest 或 probe
- **THEN** 该输出使用 owner-specific output mode
- **THEN** 该输出使用 owner-specific framing
- **THEN** 该输出使用 owner-specific orchestration
- **THEN** 可复用 `docnav-json-io` 或 diagnostics helper 的前提是保持既有 schema、plain text 或 stderr boundary
