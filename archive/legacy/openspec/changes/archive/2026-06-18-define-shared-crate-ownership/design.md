本 design 定义 Docnav 共享 Rust crate 的实现分层、依赖方向、文档先行门禁和迁移顺序，确保去重不改变现有 public contract。

## 背景

Docnav 已经拆分原始协议、阅读输出、core CLI、adapter SDK 和 Markdown adapter 的职责，但部分横切流程仍在多个 crate 中重复表达：

- core CLI 和 adapter direct CLI 都需要兼容 unknown flag、extra positional 和 unused operation flag。
- readable-json、readable-view、protocol-json 和 adapter direct machine command 都需要稳定 warning 承载边界。
- core CLI 和 adapter SDK 都需要从 operation result 或 stable error 派生 surface 输出。
- Markdown adapter 中的 entry/text 分页算法是后续 adapter 可复用的字符预算逻辑，但不应携带 Markdown ref 或 heading 语义。
- protocol request/response、manifest 和 probe 都存在 `serde_json::Value` 到 schema 校验、typed deserialize 和 semantic validate 的重复流程。

本设计只上移稳定契约和机械流程，owner-specific 语义判断留在原 owner。每个共享 crate 只拥有可复用的机械流程或稳定契约，调用方继续拥有 surface policy。

迁移顺序必须文档先行：进入实现时，先同步主规范、schema/example/fixture/testing 文档，再进入代码迁移。这样后续实现可以按 owner 和 contract 执行，避免代码先行造成 owner、schema 或输出契约漂移。

## 目标 / 非目标

**目标：**

- 定义 `docnav-diagnostics`、`docnav-cli-args`、`docnav-json-io` 和 `docnav-output` 的职责、依赖方向和 public surface。
- 定义 `docnav-readable` 作为 `docnav-output` 下层 helper 的职责收敛方式。
- 定义 `docnav-protocol` decode/validation helper、request id helper 和 `docnav-adapter-sdk` paging helper 的边界。
- 明确实现前必须先同步主规范和验证材料。
- 保持当前 CLI flags、输出字段、schema shape、warning envelope、error code、exit code 含义和 pagination 语义不变。
- 为后续实现提供小步迁移顺序和验证门禁。

**范围边界：**

- Path display normalization 继续由 core 内部拥有；本 change 不新增 path utility crate。
- Adapter process startup、registry command path 和 stdout/stderr collection 继续由 core runtime 拥有；本 change 不新增 process runner crate。
- Manifest/probe/invoke 的边界校验保持在 protocol、adapter SDK 和 core 各自 owner 下；本 change 不新增 adapter boundary crate。
- `docnav-output` 只拥有 document operation output orchestration，不拥有 manifest/probe schema、adapter routing、format detection、ref parsing、Markdown navigation 或 MCP transport。
- `docnav-json-io` 只拥有低层 JSON IO，不拥有 schema、protocol/readable wrapper、warning、stderr 文案、output mode 或 exit code policy。
- `docnav-mcp` 继续保持 thin bridge 定位。

## 决策

### 决策：`docnav-output` 只拥有 document output 编排

`docnav-output` MUST 位于 `docnav-readable` 和 `docnav-json-io` 之上、core CLI 和 adapter SDK 之下。它拥有 document operation 的 `readable-view`、`readable-json` 和 `protocol-json` 输出模式分流，包括 warning 注入、readable/protocol 选择和 stdout/stderr 通道决策。它 SHOULD 暴露一个 document-only 的窄 facade，由调用方传入 output mode、operation、request id、document outcome 和 warnings；调用方继续拥有业务 outcome 构造、mode 选择和最终 exit code decision。

理由：output orchestration 是跨 surface 行为，而 readable-view rendering 只是其中一种渲染策略。把编排放在 readable 之上，可以让 `docnav-readable` 更小，也让 document output mode 的分流只存在一种实现。

备选方案：把所有 output 行为继续放在 `docnav-readable`。不采用，因为 protocol-json stdout 纯度、stderr diagnostics 和 JSON writer failure 都不是 readable renderer 职责。

### 决策：`docnav-json-io` 只拥有低层 JSON 写出

`docnav-json-io` MUST 位于 `docnav-output`、core CLI 和 adapter SDK 的下层。它只拥有 JSON value serialization、newline writing 和 serialization/write failure plumbing。它 MUST NOT 拥有 schema validation、protocol envelope、readable payload、warning envelope、stderr diagnostic text、output mode dispatch、manifest/probe semantics 或 exit code policy。

理由：JSON 写出 mechanics 会被 document output、core 非文档 machine output 和 adapter direct machine output 复用，但这些 surface 的 wrapper、schema 和错误归属不同。把它拆成下层窄 crate，可以避免 `docnav-output` 变成泛化 output crate。

备选方案：把 JSON writer helper 放进 `docnav-output`。不采用，因为 help、version、manifest 和 probe 不属于 document output mode，却可能需要同一低层 JSON 写出 mechanics；让这些非文档 surface 依赖 `docnav-output` 会模糊 owner 边界。

### 决策：`docnav-readable` 保留为 readable helper

`docnav-readable` MUST 继续拥有 typed readable payload/value helper、`ReadableViewKind`、renderer config、readable-view block rendering 和 conformance vectors。它 MUST NOT 拥有 protocol response envelope、output mode 选择、stderr warning rendering、exit code mapping 或 adapter direct CLI command behavior。

理由：readable-view block framing 有独立稳定契约和跨语言 conformance。让它保持聚焦，可以降低 churn，也让 `docnav-output` 只把它当成一个输出目标调用。

备选方案：把 readable-view renderer 合并进 `docnav-output`。不采用，因为这会把 renderer conformance 和 CLI surface dispatch 混在一起，也会增加非 Rust renderer consumer 的理解成本。

### 决策：diagnostics 和 argv compatibility 分别成 crate

`docnav-diagnostics` 拥有稳定 warning envelope type、`WarningId` opaque newtype、共享 warning id 常量、warning id/effect/details、argv warning constructor 和 stderr warning text formatting。`WarningId` 的 wire contract 仍是可扩展字符串；`docnav-diagnostics` 只拥有共享 id 和格式校验，不枚举所有未来 adapter-owned warning id。`docnav-cli-args` 拥有直接 CLI 的 loose token scanning，输入由调用方提供 command context 和 known value flag metadata。两个 crate 都不解析业务参数，也不拥有 `StableError`。

理由：warning shape 属于 diagnostics contract，loose argv scanning 属于 direct CLI compatibility；二者都被 core CLI 与 adapter direct CLI 共同使用。

备选方案：把 warning 放入 `docnav-output`，把 argv scanning 留在 core/SDK。不采用，因为前者会让 protocol-json stderr 和 machine command warning 难以复用，后者会继续保留两套 loose argv 规则。另一个备选方案是将 warning id 做成普通 enum；不采用，因为后续 adapter-owned id 需要低成本扩展，且新增 enum variant 会增加 Rust exhaustive match 的兼容成本。

### 决策：protocol helper 保持 protocol-shaped，错误归属留给调用方

`docnav-protocol` 增加 `serde_json::Value -> schema validate -> deserialize -> semantic validate` 的机械 pipeline helper。helper 返回 typed success 或 validation facts/errors，让 core、SDK 或 tests 继续保持既有 error category、field path、stderr/readable text 和 exit behavior。

理由：protocol shape 和 semantic validation 是共享规则，但 core invoke validation、adapter SDK invoke stdin、manifest/probe validation 和测试的 process-boundary error ownership 不同。

备选方案：暴露一个直接返回 `StableError` 的高层 decode function。不采用，因为这会合并不同 surface 的错误归属，导致错误 attribution 难以维护。

### 决策：paging helper 保持低层且 format-neutral

`docnav-adapter-sdk` 承接 character budget、entry/text pagination、next page calculation 和 entry truncation helper。helper 消费已经构造好的 generic entries 或 text payload，不生成 ref、不检查 Markdown heading，也不定义 adapter navigation semantics。

理由：pagination mechanics 可以跨 adapter 复用；navigation strategy 和 ref semantics 必须由 adapter 拥有。

备选方案：新增单独 paging crate。暂不采用，因为 adapter SDK 已经是 adapter-facing shared layer，目前没有非 adapter consumer 需要这个 helper。

### 决策：第二梯队 helper 保持窄边界

exit code mapping helper 只把 `StableErrorCode` 分类为可复用类别，core/adapter 的 concrete exit code enum 继续各自拥有。request id helper 只拥有公共 id format 和 fallback generation，不拥有 tracing 或 logging。

理由：这些 helper 能删除低层重复分支，但不应把 surface-specific policy 移入共享 crate。

## 风险 / 取舍

- 风险：`docnav-output` 变成泛化 output 大 crate。缓解：spec 明确它只拥有 document operation output orchestration；低层 JSON 写出拆入 `docnav-json-io`，manifest、probe、help、version 不归 `docnav-output` 定义。
- 风险：`docnav-output` 和 `docnav-adapter-sdk` 形成依赖环。缓解：`docnav-output` 只依赖 `docnav-protocol`、`docnav-readable`、`docnav-diagnostics` 和 `docnav-json-io`；SDK 和 core 依赖 `docnav-output`。
- 风险：warning exact text 或 token grouping 被误判为稳定契约。缓解：测试只断言 stable envelope、id/effect/details family fields 和通道位置，不断言 exact reason 或 token grouping。
- 风险：warning id 自由字符串漂移。缓解：`WarningId` 使用 opaque newtype、共享常量和格式校验；消费者必须容忍未知 id，新增可观察 id 时同步 docs/example/fixture 或 characterization test。
- 风险：protocol decode helper 改变错误顺序。缓解：先写现有 invalid JSON、schema invalid 和 semantic invalid 路径的 characterization tests，再引入 helper。
- 风险：paging helper 泄漏 Markdown-specific behavior。缓解：helper API 使用 generic entries/text；测试覆盖 Unicode、oversized entry 和 next page，不依赖 Markdown ref。
- 风险：代码先行导致主规范追不上实现。缓解：tasks 把主规范和验证材料同步放在代码迁移之前，并将 crate 迁移排在 characterization tests 之后。

## 迁移计划

1. 先同步主规范、schema/example/fixture/testing 文档，明确共享 crate ownership、输出边界、argv 边界、协议解码和分页 helper 规则。
2. 为当前 warning、loose argv、output mode、protocol-json stderr、paging 和 decode failure behavior 增加 characterization tests。
3. 新增 `docnav-diagnostics`，迁移 core 和 adapter SDK 中的 warning envelope 与 stderr warning formatting。
4. 新增 `docnav-cli-args`，迁移直接 CLI loose token scanning，保留调用方 typed argument validation。
5. 新增 `docnav-json-io`，迁移低层 JSON value serialization、newline writing 和 serialization/write failure plumbing。
6. 新增 `docnav-output`，放在 `docnav-readable` 和 `docnav-json-io` 之上；迁移 core CLI document output dispatch 和 adapter SDK direct document output dispatch。
7. 将 Markdown adapter pagination mechanics 上移到 `docnav-adapter-sdk` paging helper；Markdown navigation/ref logic 继续留在 `docnav-markdown`。
8. 增加 `docnav-protocol` decode/validation 和 request id helper，迁移重复 decode 与 id generation 路径。
9. 增加第二梯队 exit classification helper，仅删除重复代码，不改变 surface policy。
10. 每个迁移 slice 运行 focused tests；最终跨 crate 交付运行 workspace verifier。

回滚策略：每个迁移 slice 必须独立可 review 和可 revert。若共享 helper 改变可观察行为，回滚该 slice，并保留暴露 drift 的 characterization tests。
