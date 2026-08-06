JSON 专用 `readable-view` 是 `add-json-adapter` 之后的独立 presentation handoff；它不重新打开已经完成的 raw adapter 验收。

## 文档状态

- 状态：`implementation-blocked`；长期产品方向由 [展示契约批准后推进 JSON 专用阅读输出](../../../docs/decisions/product-direction/advance-json-readable-presentation-after-contract-approval.md)拥有。
- Current：generic `readable-view` 已验收；JSON 专用 presentation 仍是 Planned target，不是 Current。
- 实施门禁：只固定已确认的 owner 与 raw/output/ref 边界；必须先按 [tasks](tasks.md) 关闭 presentation contract 与 renderer-selection 开放问题，才能修改 owner、测试或实现。

## Why

JSON adapter 已通过现有 generic `readable-view` 完成 raw adapter 验收；该 renderer 能承载 JSON raw facts 和可继续使用的 opaque ref，但不构成经批准的 JSON 专用 presentation。长期输出方向要求格式专用展示继续留在 `readable-view`，因此需要一个独立后续 change 决定并交付 JSON presentation，而不是把它作为 `add-json-adapter` 的前置条件或未完成项。

## What Changes

- **Current:** `docnav-json` 的当前选择链路、outline、read、find、info 和 full-read 已走通现有 generic `readable-view` 与 `protocol-json`。当前代码、测试和 release artifact 只证明该 generic presentation；本 change 的 target 不是 Current。
- **Target:** 在既有 `readable-view` output path 内提供 output-owned 的 JSON 专用 presentation。该 presentation 只消费 navigation 形成的同一个 immutable `ProtocolResponse` 和其中已有的 adapter raw facts。
- **Contract gate:** 在实施前明确每个适用 operation/branch 的信息密度与输出 shape、JSON 标点与 escaping、完整 opaque ref 的路径定位信号、preview 来源与边界、分页显示以及 renderer selection mechanics。具体选择记录到 design 的 Decisions 和本 change 的 delta spec 后，才能开始实现。
- **Boundary:** 保持 `protocol-json`、`ProtocolResponse`、JSON adapter result、ref、ordering、cost、page、schema/example shape 和 public output values 不变。Renderer 不解析 opaque ref，也不从 ref 合成 hierarchy、depth、parent 或 indentation。
- **Evidence:** 用 output contract tests、真实 `docnav` CLI、canonical package smoke 和同源 raw/readable parity 证明最终批准的 presentation；在代码证据齐备前，owner 文档不得把 target 标为 Current。
- **Independent sequencing:** Invocation-private document state 已属于 Current 输入基线；`redesign-token-cost-estimation`、`redesign-find-result-model` 和 `audit-runtime-performance-boundaries` 不是本 change 的前置，其未决语义不得被复制或预选。若其它 workstream 在实施前改变 Current 输入事实，只在门禁审计中按新基线重核。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `output-contract`: 为既有 `readable-view` 增加 JSON 格式专用 presentation 的目标边界、批准门禁和验证要求，同时保持 raw protocol isolation。

## Impact

- 长期行为 owner 是 `docs/output.md`；`docs/adapters/json.md` 继续拥有 JSON raw facts、ref 和 navigation semantics。本 change artifacts 只拥有这次 handoff 的目标、已确认边界、开放问题、任务和验收依据。
- 门禁关闭后，预期影响 core document output composition、output-owned renderer 实现、output/core tests、真实 CLI smoke、release-package smoke 和相应 Case 证据。具体 renderer selection 接入点与 presentation shape 由 design 开放问题决定，本文不预选。
- `add-json-adapter` 是已完成的 Current 基线，不在本 change 的可修改范围内，也不因本 change 尚未实施而失去验收状态。
- 本 change 不新增 public output mode、adapter-owned presentation、serialized renderer id、readable machine schema或用户配置面；内部 selection mechanics 由 design 门禁决定，不在 proposal 中预选。
