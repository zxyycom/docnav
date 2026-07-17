## 1. Confirm Scope and Baseline

- [x] 1.1 确认主目标为 core-owned input pipeline：format-specific 参数可存在，core 是唯一 declaration owner，adapter strategy 只消费 standard operation input。
- [x] 1.2 审计 `AdapterOptionSpec`、static registry、dynamic CLI catalog、navigation aggregation 与 `NativeOptionHandoff` 的生产调用链。
- [x] 1.3 保存 typed-fields、resolution companions 与 navigation 的测试基线，并在 evidence report 记录既有 `processing_id_compile` trybuild snapshot mismatch。
- [x] 1.4 补充 adapter-contracts、Markdown、docnav CLI、protocol contract validation 与 workspace dependency baseline。

## 2. Characterize Preserved Behavior

- [x] 2.1 覆盖 `--max-heading-level` 与 `options.docnav-markdown.max_heading_level` 的 accepted operations、range、default、project/user precedence、selected/unselected namespace、config inspect、Markdown outline/find effect、adapter-side validation、protocol options parity 与现有 diagnostic golden。
- [x] 2.2 固定 `Replace`、`Append`、`MapMerge`、`DenyConflict`、priority/tie-break、invalid candidate、default fallback、materialization 与 provenance tests。
- [x] 2.3 固定 env locator/`extract_env` 的 declared-only extraction、missing、decode failure 与 source attribution；证明 locator absence 不产生 env candidate，当前未启用字段保持 `explicit > project > user > built_in`。
- [x] 2.4 证明 protocol contract validation 可通过 direct `FieldDefSetBuilder` 完成 scalar 与 compound field validation。
- [x] 2.5 用 focused fixtures 区分三类行为：core 无法 materialize 时阻止 dispatch、well-typed adapter semantic failure 由 strategy 返回、同一规则在 core/adapter 重复校验时接受域和 diagnostic mapping 一致。

## 3. Checkpoint A: Build the Core-Owned Input Pipeline

- [x] 3.1 在一个 core-owned module 中声明 `page`、`limit`、`pagination.enabled`、`output` 与 Markdown `max_heading_level`；每项包含 identity、已启用的 CLI/env/config locators、standard value kind、default、merge strategy、operation binding、closed compile-time consumer binding、可选的精确 static adapter-id 标记，以及 core 选择执行的 validation facts。Strategy-visible `page` / `limit` / `max_heading_level` target shared `StandardInputBinding`；pagination/output target navigation/core-owned closed variants。未标记参数为 common，env locator presence 表示该字段启用 env，且 catalog 不吸收 adapter routing、path/ref/query、`invocation_log`、config-path flags 或 protocol/manifest/result/private-state fields。
- [x] 3.2 在现有 shared operation-contract 层定义 operation-specific closed typed input 和 strategy signature；每个策略可见值使用静态字段、typed accessor 或 closed variant，navigation 负责构造，adapter 不使用通用 parameter lookup 或 protocol request 取得调用数据，并且仍可执行或重复执行 semantic validation。
- [x] 3.3 在 catalog 定义 Markdown `max_heading_level` 并标记为 `docnav-markdown`；让 catalog construction 拒绝 duplicate identity/locator、标记 unknown adapter id、missing or incompatible consumer binding、invalid operation binding 与 incompatible definition。
- [x] 3.4 让 document CLI parser、config inspect 与 runtime config validation 从完整 scalar catalog 构造 config-validation projection，并与既有 `outline.mode_rules` / `outline.thresholds` owner-specific compound validator 组合，删除 registry-backed `NativeOptionCatalog` discovery；config inspect 只承诺 catalog/core 与该 compound supplement 覆盖的检查，不要求 compound algorithm 变成普通 scalar field。
- [x] 3.5 让 navigation 从同一 catalog 派生 selected-operation field set：先保留 untagged 参数与 `adapter_id == selected_adapter_id` 的参数，再按 operation binding 过滤后进行 candidate extraction/resolution；其它已知 adapter 的合法 source facts 不进入本次 input，并删除 `OperationFieldSet` 的 adapter-option side channel。
- [x] 3.6 从同一 resolution result 按 closed consumer binding 分别构造 protocol `Options`、strategy-visible standard operation input 与 `PreparedNavigationRequest` / core output projection；将 `pagination.enabled` 与 `limit` 归一化为 effective limit，保证 `output` 不进入 adapter input；删除 `OptionEntry` metadata sidecar 与 `NativeOptionHandoff` / `NativeOptionValue`，同时保持 field/config/`option_issues` diagnostic parity。
- [x] 3.7 让 adapter definition 只提供 routing、capability 与 strategy facts，让 Markdown strategy 只通过 standard input 使用 `max_heading_level`；删除 `AdapterOptionSpec` authoring、declaration conflicts、registry discovery、selected-adapter registration 以及无剩余 consumer 的 native-option diagnostic glue。
- [x] 3.8 通过 product parity、full-validation/selected-operation view、known-other/unknown adapter path、untagged/matching/non-matching tag、env locator absent/present priority、protocol options、closed standard-input construction 与 layered-validation focused tests，并用限定路径搜索证明 runtime 只有 catalog definition source、adapter 不处理 raw sources、protocol `Options` 或通用 parameter bag，不存在第二条 declaration/discovery/handoff 路径。

## 4. Checkpoint B: Perform Secondary Infrastructure Cleanup

- [x] 4.1 收敛 adapter definition：让 `Adapter` trait 只保留 probe、固定 operation strategies 与已支持 hooks，让 `AdapterDefinition` 直接组合 manifest identity、capabilities 与 strategy；删除无独立行为的 definition builder/transition、parallel handler declaration/support 与 `FullReadCapabilityGroup`，同时保持 capability validation 与 operation dispatch。
- [x] 4.2 收敛 registry/routing representation：删除只转发单一静态事实的 `NavigationAdapterRef` 与 `AdapterRecord`，移除 no-op registry loader 和 per-record `core_static` storage，同时保持 multi-adapter routing、adapter-list 与 doctor behavior。
- [x] 4.3 让 processing metadata 组合 canonical field facts 与 locator/input-kind context，删除 schema/processing views 的重复 facts storage。
- [x] 4.4 将 navigation direct input 的唯一 production `SourceKind::Custom` / `SourceLocator::Custom` 用途替换为固定 `Direct` kind 与 typed direct-path locator；删除开放式 custom string validation/errors，并证明没有其它 production consumer。
- [x] 4.5 移除 public foreign `FieldDefDeclaration` injection 与无 production consumer 的 `FieldDefs` derive/trait glue、proc-macro package、fixtures 和 facade exports；保留 direct builders。
- [x] 4.6 删除无 production consumer 的 `cli-config-resolution-clap` package、examples/tests/workspace entries；core 私有 mapping 只覆盖当前 catalog 参数。
- [x] 4.7 更新 retained adapter/typed-fields/resolution facades 与 README，明确保留 strategy、routing、capability、resolution core、Serde companion、env extractor、layered validation、defaults、materialization 与 provenance。
- [x] 4.8 运行 adapter-contracts、routing、typed-fields、resolution、Serde、env 与 protocol focused tests，并用 dependency/search gate 证明每个删除项无剩余 consumer、四种 merge strategy 仍存在且 Checkpoint A behavior 未改变。

## 5. Sync Contract Owners

- [x] 5.1 更新 architecture、adapter contract、navigation input resolution 与 Markdown adapter 主规范，明确 core declaration、full config validation/selected-operation resolution、closed standard-input construction、env activation 与 adapter strategy/semantic-validation boundary。
- [x] 5.2 更新 testing case ledger、`@case` markers、受影响 crate README、examples 与 API docs，只保留仍存在的 public/runtime surface。
- [x] 5.3 核对 schemas、examples、protocol、raw/readable output、diagnostics、refs、pagination 与 release materials；同步 ownership 文案并证明 observable shape 未改变。

## 6. Verify and Close

- [x] 6.1 运行 adapter-contracts、Markdown、navigation、docnav CLI、typed-fields、resolution/Serde/env、layered validation 与 protocol focused tests。
- [x] 6.2 运行 `bun run verify:docnav-workspace`；相对 1.3/1.4 baseline 不得新增失败，既有异常必须在 closeout 中列出。
- [x] 6.3 运行 `bun run validate:docs`、`bun run validate:openspec`、strict change validation 与 `git diff --check`。
- [x] 6.4 限定路径搜索确认 Checkpoint A 的旧 parameter path 与 Checkpoint B 的已批准删除项均已清除，同时 standard input、strategy validation、catalog、routing、capability、merge、env、Serde、protocol validation 与 multi-source surface 均存在。
