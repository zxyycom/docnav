本 change 定义 Docnav 共享 Rust crate 的职责、所有权、依赖方向和文档先行迁移顺序，目标是在不改变现有契约的前提下降低重复实现。

## Why

当前 core CLI、adapter SDK、Markdown adapter 和 readable 输出路径中重复实现了同类机械流程：warning envelope 与 stderr 文本、宽松 argv 扫描、document output 分流、分页截断、协议解码校验、exit code 分类、JSON 写出和 request id 生成。继续分散实现会让 CLI、adapter direct CLI、protocol-json、readable-json 和 readable-view 的兼容边界更难保持一致。

这类改造会触碰 public contract、crate ownership、直接 CLI、adapter SDK、protocol 和 readable 输出边界。必须文档先行：先通过 OpenSpec、主规范和验证材料定义合法边界，再迁移代码；否则实现过程中容易反复发现代码与规范不一致。

## What Changes

- 新增共享 crate：`docnav-diagnostics`、`docnav-cli-args`、`docnav-json-io` 和 `docnav-output`。
- 调整既有共享 crate：`docnav-readable` 收敛为 readable payload 与 readable-view renderer helper；`docnav-protocol` 补充协议解码/校验 pipeline helper 与 request id helper；`docnav-adapter-sdk` 承接可跨 adapter 复用的 paging helper。
- `docnav-output` 作为 document operation 输出编排 owner，负责 `readable-view`、`readable-json`、`protocol-json` 的模式选择、warning 注入、protocol/readable 包装和 stdout/stderr 分流；help、version、manifest 和 probe 保持在非文档输出边界外。
- `docnav-json-io` 作为低层 JSON IO owner，负责 JSON value serialization、newline writing 和 serialization/write failure plumbing；schema、protocol envelope、readable payload、warning、stderr 文案、output mode 和 exit code policy 仍由各 surface owner 决定。
- `docnav-diagnostics` 作为 diagnostics owner，负责稳定 warning envelope、`WarningId` newtype 与共享常量、warning id/effect/details、argv warning 构造器和 stderr warning 文本格式；`StableError`、exit code enum 和 adapter 选择语义仍由原 owner 决定。
- `docnav-cli-args` 作为 direct CLI argv compatibility owner，只处理宽松 argv token 扫描；业务参数解析仍由调用方负责，adapter `invoke` stdin JSON 继续走严格 protocol decoding。
- `docnav-adapter-sdk` paging helper 抽取字符预算、entry/text 分页、next page 和截断算法；Markdown ref、heading、display 和 parser 语义仍留在 `docnav-markdown`。
- `docnav-protocol` decode/validation helper 执行 `Value -> schema validate -> deserialize -> semantic validate` 的通用 pipeline；调用方继续决定错误归属、field path、stderr/readable 文案和 surface exit code。
- 第二梯队收敛为 helper：`StableErrorCode` 分类到 surface exit code 的骨架、request id 生成函数。
- 文档先行顺序：进入实现时，先同步主规范、schema/example/fixture/testing 文档，再迁移实现代码。
- 范围边界：本 change 不新增 path utility crate、process runner crate 或 adapter boundary crate；不改变 CLI flag、输出字段、schema shape、error code、warning id 或 pagination 语义。

## Capabilities

### New Capabilities

- 无。该 change 不引入新的长期产品能力，只重新定义现有共享实现和 contract owner。

### Modified Capabilities

- `docnav-contracts`: 明确共享 crate owner、依赖方向、文档先行顺序和不可上移的边界，保持协议层、阅读层和 adapter-owned 语义分离。
- `core-cli`: 明确 core CLI 通过共享 diagnostics、argv、JSON IO 和 output helper 复用直接 CLI warning、document output dispatch、JSON 写出、request id 和 exit mapping 分类。
- `adapter-protocol`: 明确 `docnav-protocol`、`docnav-adapter-sdk` 和 adapter direct CLI 的共享 helper 边界，包括 invoke 严格校验、direct CLI 宽松 argv、paging helper 和 protocol decode pipeline。
- `readable-view-output`: 明确 `docnav-output` 作为 document output orchestration 上层 owner，`docnav-readable` 保留 readable payload 和 readable-view renderer helper 职责。

## Impact

- 受影响 crate：`crates/docnav`、`crates/docnav-adapter-sdk`、`crates/docnav-markdown`、`crates/docnav-protocol`、`crates/docnav-readable`，以及计划新增的 `crates/docnav-diagnostics`、`crates/docnav-cli-args`、`crates/docnav-json-io`、`crates/docnav-output`。
- 受影响 public surface：直接 CLI warning 行为、readable warning 放置、protocol-json stdout 纯度、adapter direct CLI 输出边界、stable error 到 exit code 的分类、request id 格式和分页 helper 行为。
- 兼容性预期：无可观察行为变化。现有输出字段、warning envelope、protocol response envelope、schema examples、exit code 含义和 Markdown navigation 语义必须保持稳定。
- 文档影响：实现前必须同步主规范和验证材料，避免代码先行导致 owner、schema 或输出契约漂移。
- 验证影响：实现必须新增或调整共享 helper 的 focused tests，并保留 core CLI 与 adapter direct CLI 的跨 surface smoke coverage。跨 crate 交付最终应运行 workspace verifier。
