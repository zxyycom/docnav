## Why

本 change 首先是一次参数 ownership 与执行链路调整，typed-field support surface 清理只是后续工作。

Docnav 的 adapter 与 core 一同静态链接、发布和注册，但当前参数链路仍由 adapter 提供调用方参数声明，再由 core 各层发现、转换和回传。这个设计让 adapter 看起来能够扩展输入契约，实际上任何新参数仍必须随 core release 协调发布。

目标模型直接承认这一事实：core 定义本 release 接受的全部调用方参数，并通过 typed-field 处理输入；adapter 只实现接收 standard operation input 的格式策略。Adapter 可以执行算法所需的语义校验，但不能借校验重新获得参数声明权。

## What Changes

- 建立 core-owned closed parameter catalog，覆盖文档操作参数的 identity、CLI/env/config locators、standard value kind、default、merge strategy、operation binding、closed compile-time consumer binding、可选的精确 adapter-id 标记，以及 core 需要执行的静态校验规则。无标记参数属于通用输入；带标记参数只属于该 static adapter。
- 从同一 catalog 派生完整 config-validation projection 和 selected-operation field set；navigation 选择 adapter 后，仅让无标记参数和精确匹配 selected adapter id/operation 的参数进入 typed-field resolution。
- 由 core 定义 operation-specific closed typed input；navigation 只把 strategy-visible catalog values 绑定到该 shared contract，selected adapter 只通过策略函数消费该输入。Pagination/output 等 core/navigation-only controls 使用各自 closed consumer binding，不进入 adapter strategy input。
- 采用分层校验：core 必须拒绝无法 materialize 为标准类型的输入；adapter-specific 语义可以只由 adapter 校验，也可以由 core 提前校验并在 adapter 中防御性重复。
- 将 Markdown `max_heading_level` 的 public input definition 和 standard-input binding 移入 catalog；Markdown 保留格式算法及其语义安全检查。
- 让 adapter definition 只提供 routing、capability 与 strategy facts，并在等价验证后收敛为唯一 catalog → typed-field → closed consumer binding 路径；其中 strategy-visible subset 继续进入 standard input → strategy。
- 主路径独立通过后，再执行维护证据支持的次要基础设施清理；该清理不得改变 catalog、standard input、source precedence、merge semantics 或 observable behavior。

Catalog 只覆盖调用方可配置的文档操作参数；protocol/manifest/probe contract-validation fields、result facts 与 adapter-private algorithm state 继续由各自 owner 定义。
本 change 的 catalog inventory 固定为 `page`、`limit`、`pagination.enabled`、`output` 和 Markdown `max_heading_level`；adapter routing、document path/ref/query、`invocation_log` 与 config-path selection flags 不进入 catalog。`pagination.enabled` 与 `limit` 归一化为 effective limit，`output` 只进入 `PreparedNavigationRequest` / core output projection。
Env locator 是逐字段 source activation fact；本 change 定义统一接入规则，不为现有产品字段新增 env surface。

## Capabilities

### New Capabilities

- 无。

### Modified Capabilities

- `docnav-architecture`：明确 core parameter authoring、navigation resolution 与 adapter behavior ownership。
- `core-cli`：增加 closed parameter catalog，并使 static adapter registry 只提供 adapter behavior facts。
- `navigation-input-resolution`：从 catalog 过滤、解析和绑定参数，构造 standard operation input。
- `adapter-contract`：strategy 只接收 standard operation input；adapter definition 不提供参数声明，但策略可校验标准值。
- `markdown-adapter`：继续使用 `max_heading_level`，但不再定义其 public input。
- `typed-fields`：保留 canonical field mechanics 和分层校验能力，收窄 construction surface 并去重 projections。
- `cli-config-resolution`：保留 multi-source resolution、Serde 与 env extraction，并将 framework-specific projection 收窄到实际消费者。

## Impact

- 代码范围：`crates/docnav`、navigation、adapter-contracts、Markdown adapter、typed-fields、CLI/config resolution 与 workspace manifests。
- 契约材料：architecture、adapter contract、navigation input resolution、Markdown adapter、typed-field/resolution docs、schemas/examples 与 case ledger。
- 产品兼容性：既有 CLI flags、config paths、source precedence、merge behavior、accepted value domain、diagnostics、protocol/readable output、adapter results、refs 与 pagination 保持不变；内部校验可以发生在 core、adapter 或两处。
- Rust API compatibility：adapter parameter authoring 与经消费者审计确认冗余的 support APIs 不保留兼容层；具体删除项和验证 gate 由 `tasks.md` 与维护证据负责。
- 有意代价：新增 adapter-scoped 参数需要在同一 core release 中协调 catalog 与 adapter consumer；未来 runtime plugins 或 independently published adapter SDK 需要独立 change。
