# Proposal

本计划在既有 `readable-view` 内交付 output-owned 的 JSON 专用 presentation；它以已确认的长期方向为输入，并把尚待批准的展示契约作为实施序列中的首个显式门禁。

## Why

JSON adapter 已通过 generic `readable-view` 暴露可继续导航的 raw facts，但 generic presentation 不等于 JSON 专用的信息密度、定位和分页体验。[展示契约批准后推进 JSON 专用阅读输出](../../docs/decisions/product-direction/advance-json-readable-presentation-after-contract-approval.md) 已经确认该里程碑及其 raw/output/ref 边界；当前 Change 需要关闭精确 presentation 与 renderer-selection 门禁，再完成实现和证据。

## Outcome

在不改变 `ProtocolResponse`、`protocol-json`、JSON raw result、opaque ref、ordering、cost、page 或 public output mode 的前提下，内置 `readable-view` 为批准范围内的 JSON operation/branch 选择并渲染格式专用 presentation；真实 CLI、canonical package 和 contract tests 共同证明它与同一 raw facts 一致。

## Scope

- 纳入 operation/branch 覆盖、稳定 display 字段与顺序、标点和 escaping、完整 opaque ref 定位信号、preview、page/continuation 以及 linked renderer selection 的批准与实现。
- Presentation 只消费 immutable `ProtocolResponse`；不得重读文档、解析 ref、调用 adapter 或把 presentation fact 写回 raw protocol。
- 不新增 output mode、serialized renderer id、用户配置面或 adapter-owned presentation。
- Token cost、find result、document state 和 runtime performance 不是统一前置；它们先落地的 Current 变化只触发范围内重核。

## Success Criteria

- 六组 presentation/selection 问题均有明确批准答案，并已进入本 design、owner 文档和可证伪测试预期。
- Output layer 实现 JSON renderer，core linked composition 按批准规则选择它，generic/其它格式与提前 failure 行为保持既有契约。
- `protocol-json` raw facts 与 JSON `readable-view` 通过 parity evidence，renderer failure 保持 stdout 为空且不 fallback。
- 真实开发 CLI、canonical release package、schema/example、Semantic Case 和完整 workspace verification 通过。

## Affected Owners

- [输出模式](../../docs/output.md)：renderer contract、selection、presentation、failure 和 output channel。
- [JSON Adapter](../../docs/adapters/json.md)：只摘要 raw/readable 边界，不拥有 presentation。
- [原始协议](../../docs/protocol.md)、`docs/schemas/`、`docs/examples/`：证明 raw shape 不变和 raw/readable mapping。
- Output/core 实现、contract/integration tests、CLI/package smoke，以及 [测试策略](../../docs/testing.md) 与对应 Semantic Cases。
