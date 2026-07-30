本 proposal 是 `replace-probe-traversal-with-inferred-routing` 的临时变更工件：它规划以一次内部格式推断和精确 registry 路由替代 adapter probe 遍历，但不在依赖与兼容性审计获人工批准前授权实施。

## Why

Current automatic selection 按 static registry 顺序逐个执行 adapter probe，并把前序候选失败当作可恢复证据；因此选择结果依赖注册顺序，每增加一个 adapter 都会扩大候选执行面，也让“格式识别”和“adapter 执行”形成两套分散入口。现有 Markdown 与 JSON 已证明 format identity 可以归一到 registry metadata，现在应把 automatic selection 收敛为一个可审计的 routing 决策，同时保持 selected adapter 对真实 parse 和 operation 语义的所有权。

## What Changes

- **BREAKING**：automatic selection 从“按 registry 顺序 probe，选择第一个 `supported: true`”改为“恰好一次内部格式推断 → project-owned normalized format identity → registry format identity 精确匹配 → dispatch”；registry 顺序不再影响格式选择。
- unknown inference 使用现有 `FORMAT_UNKNOWN` + `FORMAT_NOT_RECOGNIZED` details；已识别但 registry 不支持的 format 使用 `FORMAT_UNKNOWN` + `NO_SUPPORTED_ADAPTER` + normalized `format`；inference 返回多个 normalized identities 时使用现有 `FORMAT_AMBIGUOUS`，其 candidates 只投影能精确映射到 registry 的 project adapter。Static registry 的 duplicate format identity 是 release-validation blocker，防御性 runtime outcome 是 global `INTERNAL_ERROR`，而不是按 registry 顺序猜 winner。
- 显式 `--adapter` 继续表达 caller intent：按 adapter id 精确 lookup，跳过 automatic inference；lookup 成功只确定 strategy，selected adapter 仍必须在真实 operation 中读取并 parse 文档。
- selected adapter 的 parse、semantic validation 或 operation failure 是该次执行的最终 adapter diagnostic；automatic 和 explicit path 都不得回退到其它 adapter。
- 不引入 custom inference trait、confidence scoring framework、adapter callback 或第二套路由注册机制。优先审计一个成熟的现有 Rust 库并在 navigation 内部直接使用，但本 proposal 不选择或批准任何 dependency。
- **BREAKING**：基于当前树没有独立 owner-backed production probe consumer 的证据，完整删除 `Adapter::probe`、`AdapterDefinition::probe`、`ProbeResult`、probe decode/runtime validation、typed-field projection、schema/examples/fixtures 和 probe-only candidate evidence。实施前的兼容性审计仍必须核实这一结论；若发现真实 consumer，必须停止并回到 artifacts 与人工批准，不能临时保留 inspection surface。
- 先完成 change-local 依赖与兼容性 audit，覆盖 ecosystem、maintenance、security、license、MSRV、支持 targets、binary/package size、startup、JSON/Markdown coverage 和 alternatives；人工批准是 Cargo、lockfile、production code 或 schema 修改的前置 gate。

## Non-Goals

- 不公开 inference enum、library error/text、confidence 或 detection trace；public contract 只观察 project-owned selection result/diagnostic。
- 不建立可插拔 detector framework、自定义评分/阈值系统、adapter-owned inference hook、fallback chain 或 content-type registry。
- 不改变 adapter-owned ref、parse、outline/read/find/info、full-read、pagination、success envelope 或 readable output 语义；selection failure 的既有 canonical diagnostic details 会按 owner delta 精确更新。
- 不实现 project-wide find；本 change 是 `add-project-wide-find` 的实施前置，只提供其 per-document routing/failure classification，不接管 project discovery/result/pagination。
- 不在本 change 中实现 document-state reuse、runtime performance audit、code adapter 或 JSON adapter 的 owner work。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `adapter-contract`: 固定 adapter surface 完整删除 probe；format descriptors 继续由 registry-facing definition 提供，core registry validation 阻断 duplicate format identity，真实 parse 仍由 selected strategy 拥有。
- `docnav-architecture`: navigation/core 私有 routing mechanism 拥有 format inference 与 selection；adapter 只拥有 selected 后的真实 parse、format semantics、ref 与 operations。
- `navigation-input-resolution`: automatic selection 改为单次内部 inference 后的精确 format lookup，explicit selection 跳过 inference，且 selected adapter failure 不再触发候选 fallback。
- `diagnostics-contract`: 复用 `FORMAT_UNKNOWN`、`FORMAT_AMBIGUOUS` 和 `INTERNAL_ERROR`，定义 mechanism-neutral reason、exact details 与 registry invariant failure。
- `protocol-contract`: 以既有 failure envelope 投影新 routing outcomes，并从 shared protocol surface 删除 probe result。
- `contract-validation`: 删除 probe JSON schema/runtime validator 及其 validation materials，不保留无 owner 的 dead validation path。
- `typed-fields`: 删除 probe consumer-local field definitions/projections，并明确 private inference outcome 不进入 typed-field catalog。
- `markdown-adapter`: 删除 Markdown-owned selection probe 义务；Markdown 只在被精确选中后执行既有 document operations。
- `json-adapter`: 删除 JSON-owned selection probe 与 post-probe reload 特例；automatic routing 只负责识别 JSON identity，selected JSON operation 继续执行完整 parse 与 JSON-owned validation。

## Impact

- 计划中的 implementation surfaces：`crates/shared/navigation` routing、`crates/shared/adapter-contracts` 固定 strategy/definition、core static registry validation/lookup、shared protocol/contract-validation/typed-field consumer、内置 adapters、diagnostic/schema/examples/fixtures 与相关 tests；probe surface 在 blocking audit 确认无真实 consumer 后整体删除。
- Dependency surface：可能新增一个格式推断 crate，但名称、版本、features 和 transitive graph 均未获批准；blocking audit 和人工 gate 通过前不得修改 Cargo manifests 或 lockfile。
- Diagnostics：navigation 产出单一 primary selection diagnostic；exact code/details 由 `diagnostics-contract` 与 `protocol-contract` delta 共同固定，且不暴露候选库类型、原文 message 或 registry-order candidate evidence。
- Cross-change handoffs：`add-project-wide-find` 的 Decisions 5/12 已把本 change 记录为 predecessor并接受 filter/local/fatal planning seam；routing task 0.6 只核对并记录该 artifact-level acceptance，不等待 project task 1.3、implementation 或 validation。`reuse-adapter-document-state` 与 `add-ast-grep-code-adapter` 是 downstream rebase consumers：本 change 记录 no-probe handoff，不要求它们先选择内部机制、依赖或完成实现；它们推进时必须按最终 Current routing contract 重写自己的旧 probe/candidate-traversal基线。只有发现 handoff 会破坏其核心目标时才升级为人工决策。`audit-runtime-performance-boundaries` 只接收 measurement handoff；archived `add-json-adapter` record 只提供 probe/TOCTOU migration input。
