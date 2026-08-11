# Core-only 参数与 Typed Fields 维护证据

## Document Role

本文保存支持本 change 取舍的现状证据，不定义产品契约或实施顺序。技术决策由 [`design.md`](design.md) 负责，可验证契约由 [`specs/`](specs/) 负责，执行出口由 [`tasks.md`](tasks.md) 负责。除 OpenSpec 必需的 `REMOVED` 条目外，旧类型、旧调用链和具体删除名称只在本文与任务清单中出现；长期文档只保留目标状态。

证据日期：2026-07-17。

## Static Deployment Evidence

- `crates/docnav/src/registry.rs` 的 built-in registry 当前只注册 `markdown_adapter_definition`。
- Adapter implementation source 是 `core_static`；adapter definition 与 core release 一同编译和发布。
- `enable-local-core-adapter-service-mode` 明确不发现、启动或回退到外部 adapter executable。
- Core CLI、config docs/schema 与 navigation 已经需要知道 public parameter 的 observable shape。

这些事实说明 adapter source 目前不能独立扩展 release 接受的参数；参数 authoring 放在 adapter 侧并不形成独立部署边界。

## Current Parameter Chain

当前唯一 production native option 是 Markdown `max_heading_level`：

| Stage | Surface | Additional work |
| --- | --- | --- |
| Adapter authoring | `AdapterOptionSpec` / builder | 声明 field facts、owner 与 operations |
| Definition/registry | `AdapterDefinition::native_options`、registry aggregation | 向 core 暴露声明 |
| CLI | `NativeOptionCatalog` | 重建 operation/flag metadata |
| Navigation | `OperationFieldSet::adapter_options` / `all_adapter_options` | 将声明注册进 canonical `FieldDefSet` |
| Request/dispatch | protocol `Options` / `OptionEntry`、`NativeOptionHandoff` | 复制 metadata 并构造第二次 handoff |
| Adapter | `*_with_native_options` + key lookup | 查找值并再次检查 identity/type/range/missing |

`NativeOptionValue` 仍携带 `serde_json::Value` 与 identity/type/source metadata，而不是最终 operation-specific Rust value。Markdown 因此会重复部分基础校验。Canonical resolution 已有足够事实直接构造 protocol projection 与 standard typed accessor。

这里的冗余是 parameter declaration、metadata copy 和 handoff 链路，不是“adapter 中出现校验”本身。目标模型允许 adapter strategy 对 standard value 执行算法语义校验或防御性重复校验；它只是不再通过校验代码向 core 声明参数、source 或 merge facts。

## Typed-Field and Resolution Consumers

- Navigation 使用 `FieldDef` / `FieldDefSet`、processing locators、constraints、defaults、validation 与 typed materialization。
- CLI/config resolution 使用 ordered `Source` / `SourceCandidate`、四种 merge strategy、priority/tie-break、defaults 与 provenance。
- Protocol contract validation 大量使用 direct `FieldDefSetBuilder` 验证 request、response、manifest 与 probe fields。
- Navigation 生产路径使用 `cli-config-resolution-serde`。
- Env extraction 已有独立 contract，且产品字段将陆续接入。

因此证据支持保留 typed-fields、resolution core、Serde companion 和 env extractor；core-only 参数 ownership 不等于删除这些基础。

## Redundant Surface Evidence

- `FieldDefDeclaration` 的跨 crate production consumer 主要是 adapter option declaration；protocol 与 navigation 已可直接使用 builders。
- `SchemaMetadataView` 与 `ProcessingMetadataView` 同时保存 identity、path、value kind、constraints、default 与 merge strategy；processing view 独有事实主要是 processing id、input kind 与 locator。
- `FieldDefs` derive/trait glue 没有 production consumer，调用点位于 typed-fields tests。
- `cli-config-resolution-clap` 没有 production consumer；Docnav core CLI 已有自己的 parser/mapping。
- `cli-config-resolution-serde` 有 production consumer，不能随 Clap companion 删除。

这些删除候选都需要在实施时再次通过 workspace dependency search 和 focused tests；“当前未发现 production consumer”不是跳过验证的授权。

## Secondary Cleanup Evidence

| Current surface | Static evidence | Change decision |
| --- | --- | --- |
| `AdapterOperationHandlers`、`REQUIRED_OPERATIONS`、builder handler registration 与 dispatch `supports()` | `Adapter` trait 已静态要求 outline/read/find/info；required list 与 trait 完全相同，dispatch 在检查后仍对同一 enum 做 match | 删除 parallel handler declaration、duplicate/missing-handler errors 与 runtime support check；strategy interface 是唯一 handler contract |
| `AdapterDefinitionBuilder`、`transition_from_adapter`、trait 上的 id/manifest/options/capability metadata methods | Production definition 只有 static Markdown factory；transition helper 只有 test caller，且 builder 的动态集合主要服务将被删除的 options/handlers | 用直接 checked `AdapterDefinition` constructor；trait 只保留 strategy/probe/hooks，manifest/capabilities 只在 definition 表示 |
| `FullReadCapabilityGroup` | 只包装一个 `UnstructuredFullReadCapabilities`，额外方法仅转发且 `has_cost_measurement_unit` 无 caller | Definition 直接保存 optional capabilities 并继续执行现有 capability validation |
| `OperationFieldSet` 的 `adapter_options` / `all_adapter_options` side channel | 去掉 adapter declaration injection 后，`OperationFieldSet` 仍作为 selected-operation attribution wrapper | 仅删除 adapter-option side channel；保留 attribution wrapper，并携带按 tag/operation 过滤后的 `FieldDefSet` |
| protocol `Options.entries` / `OptionEntry` 与 `NativeOptionHandoff` / `NativeOptionValue` | `Options.entries` 不序列化；handoff 又把同一 identity/type/source/value metadata 复制一次；standard input 可直接从 resolution 构造 | `Options` 收窄为原有 values object；删除 metadata sidecar 与 native-option handoff/value，保留 wire shape 和 diagnostic parity |
| spec-derived `NativeOptionIssue` glue | `NativeOptionIssue` 与 `AdapterError::native_option_invalid` helper 仍被 strategy semantic validation / diagnostic mapping 实际消费；可删除范围仅是旧 option-spec-derived glue | 保留 issue/helper 及 observable details，只删除 spec-derived glue |
| `NavigationAdapterRef` | 只包装一个 `AdapterDefinition` 并转发 `id()` | Routing/selection 直接携带 `AdapterDefinition`；保留有 production/test value 的 registry trait |
| `AdapterRecord`、`AdapterRegistry::load(ProjectContext)` 与 per-record `implementation_source` | 删除 source 字段后 record 只包装一个 definition factory；`load` 忽略 project 且只返回 `builtin()`；所有 built-ins 与 registry output 都是 `core_static` | Registry 直接保存 definition factories；core 直接使用 static registry；implementation source 作为 registry-level fact，不在每条 record 重复 |
| `SourceKind::Custom(String)` / `SourceLocator::Custom(String)` | 唯一 production 用途是 navigation 的预解析 direct input；其余命中为 tests | 用固定 `Direct` source kind 与 typed direct path locator，删除未使用的开放式 source-name 扩展和相关 string validation |

上述项目不是新的架构主线；它们是 core-only 决策成立后可直接消失的第二份表示。实施仍以 parameter catalog → typed resolution → standard operation input → strategy 为主路径。

### Retained Boundaries

- 保留 `Adapter` strategy trait：它是多个静态 adapter 实现共享的固定行为契约，不再承担 metadata authoring。
- 保留 `NavigationAdapterRegistry` 和 routing：多 adapter 选择、probe 与测试替身是真实消费者；只删除一字段 ref wrapper 与假动态加载。
- 保留 `FieldDef` / `FieldDefSet` direct builders、四种 merge strategy、processing locators、Serde/env extraction、materialization 与 provenance；这些都有 production behavior 或 contract-validation consumer。
- 不因当前只有 Markdown 就把 registry、format selection 或 capability validation 硬编码到单一 adapter。

## Baseline Status

Task 1.3 已完成 typed-fields、resolution companions 与 navigation baseline，并记录一个变更前异常：`processing_id_compile` trybuild snapshot mismatch。后续验证必须将它作为 pre-existing failure 单独报告，不能归因于本 change，也不能用它掩盖新增失败。

Task 1.4 的 baseline 固定在 commit `0a56f5f3176242d1ef2e90523cb276c6f4c3600b`，使用 `rustc 1.97.0 (2d8144b78 2026-07-07)` 与 `cargo 1.97.0 (c980f4866 2026-06-30)`；执行前后工作树均为 clean。

| Surface | Reproducible command | Baseline result |
| --- | --- | --- |
| Adapter contracts | `cargo test --locked -p docnav-adapter-contracts --quiet` | PASS：12 passed，0 failed；doc tests 0。 |
| Markdown adapter | `cargo test --locked -p docnav-markdown --quiet` | PASS：crate tests 21 passed，integration tests 27 passed，合计 48 passed、0 failed；doc tests 0。 |
| Docnav CLI/core | `cargo test --locked -p docnav --quiet` | PASS：103 passed，0 failed；其它 test targets 与 doc tests 为 0。 |
| Protocol contract validation | `cargo test --locked -p docnav-protocol --quiet` | PASS：24 passed，0 failed；其中 request/response/manifest/probe 的 contract、decode 与 public-schema 代表测试均通过；doc tests 0。测试清单可用 `cargo test --locked -p docnav-protocol -- --list` 重放。 |
| Workspace dependencies | `cargo tree --locked --workspace --edges all --prefix depth` | PASS：完整 workspace normal/build/dev dependency 与 feature graph 可解析，lockfile 无漂移。 |

Dependency baseline 的 change-relevant normal edges 另用 `cargo tree --locked --workspace --edges normal --invert <package> --prefix none` 核对：

- `cli-config-resolution-clap` 的 normal consumer 只有 `docnav`。
- `docnav-typed-fields-macros` 的直接 normal consumer 是 `docnav-typed-fields`；后者继续被 resolution、protocol、adapter-contracts、navigation 与 docnav 路径消费。
- `cli-config-resolution-serde` 的直接 normal consumer 是 `docnav-navigation`，并经 navigation 进入 `docnav`。
- `docnav-protocol` 继续依赖 `docnav-typed-fields`；`docnav-adapter-contracts` 继续依赖 protocol 与 typed-fields；Markdown 继续依赖 adapter-contracts 与 protocol；docnav 继续组合这些 runtime crates。

Task 1.4 的五个 focused baseline 全部通过，没有新增失败。Task 1.3 已记录的 `processing_id_compile` trybuild snapshot mismatch 不在本组 focused command 的执行范围内，本 slice 未重跑、修复或重新归类该既有异常。

## Alternatives

| Option | Evidence-based result | Decision support |
| --- | --- | --- |
| 保持 adapter-owned declarations | 仍需随 core 发布，且保留完整转换链 | 不形成有效扩展边界 |
| Core catalog + standard adapter input | 与 release owner 对齐，可删除 declaration/discovery/handoff，并保留 strategy semantic validation | 支持 Checkpoint A |
| 删除全部 typed-field/resolution | 会失去已有 protocol validation、merge、multi-source、env 与 provenance 能力 | 证据不支持 |
| 在 core-only 切换后清理冗余 support surface | 删除候选有独立消费者证据和验证 gate | 支持独立 Checkpoint B |

## Evidence Limits

本审计能证明当前 static release model 中没有独立 adapter parameter authoring boundary，也能识别现有生产消费者和删除候选。

它不能证明未来永远不需要 runtime plugins、外部 adapter SDK、新 value kind 或新的 merge semantics。若产品选择这些方向，应由新的 deployment/compatibility capability 提供扩展边界，而不是让当前 change 预留未被使用的动态链路。
