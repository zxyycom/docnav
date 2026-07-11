本 design 是 `create-universal-cli-config-crate` 的 change-local 技术方案：说明如何以 Docnav 现有 `FieldDef` / `FieldDefSet` 为 canonical 标准参数模型，形成可独立维护的 Rust CLI/config resolution Cargo workspace。当前文档只存在于 `openspec/changes/create-universal-cli-config-crate/`，不直接修改主规范。

## Context

Docnav 的 `docnav-typed-fields` 已经拥有字段 identity、processing metadata、类型、约束、静态默认值、校验事实、set 构建检查和 typed materialization；`parameter-resolution` 及第一轮 `cli-config-resolution` 实现则证明了 ordered sources、merge 与 provenance 的需求。

第一轮实现完成并通过验证，但同时建立了 `FieldContract` / `FieldSet` 以及平行的 value、constraint、default 和 validation 类型。Docnav 因而需要先把 canonical typed-fields metadata 转成第二套字段模型，再手工生成 candidates。该路径能运行，却没有兑现“复用现有标准参数对象”的主要目标，也让 companion crate 的正常使用路径比必要的更复杂。

本轮允许微调 API 和实现任务。收敛后的边界是：typed-fields 拥有参数定义、类型、约束、默认值、merge strategy、校验和 materialization；resolution 拥有来源、优先级、merge 执行语义与 provenance；source-specific extractor 拥有 CLI/env/config locator 到 canonical field identity 的映射。

## Goals / Non-Goals

**Goals:**

- 让现有 `FieldDef` / `FieldDefSet` 直接成为 canonical `Parameter` / `ParameterSet`，并由每个 `FieldDef` 直接声明 `MergeStrategy`，不要求消费者维护第二套 field contract 或 merge policy table。
- 通过现有 processing 机制显式声明 CLI flag、env var 和 config path extraction strategy。
- 为 CLI、env、config、default 和 custom inputs 提供统一 candidate/source contract，并按确定 priority 与 field-level merge policy 解析。
- 复用 canonical 类型、约束和默认值完成输入校验与最终 typed materialization，同时保留 deterministic provenance trace。
- 让 Docnav 通过同一 public API 完成 hard cutover，不再构造 `generic_field_set` 或手工复制字段 metadata。
- 建立可独立 checkout、build 和 test 的 Cargo workspace 子仓库，供其它 Rust CLI 复用。

**Non-Goals:**

- 不实现新的 Docnav 文档导航能力，也不改变 protocol、adapter、operation、diagnostic code 或 output behavior。
- 不为 env/config 未声明项增加全量扫描、unused-key diagnostics 或通用 unknown-input policy。
- 不把 clap 或 structured-config framework 依赖放进 framework-independent resolution core。
- 不扩展与当前来源、校验、合并和 provenance 主流程无关的状态模型。
- 不在本 change 中执行 crates.io 发布。

## Decisions

Decisions 1-8 记录第一轮实现时的选择和验证背景。Decision 9 及之后是 2026-07-10 用户确认后的当前执行契约；发生冲突时，以编号更后的 Decision 为准。

### Decision 1: Workspace-first 的通用 crate 边界

第一轮先在 Docnav workspace 中建立 `cli-config-resolution`、`cli-config-resolution-clap` 和 `cli-config-resolution-serde` package，确保 tests、docs 和 Docnav hard cutover validation 能复用当前验证链路；长期目标是迁移到独立 repository。

### Decision 2: 字段事实与 source policy 分层（由 Decision 9 收敛）

第一轮把 stable identity、value kind、constraint、default、projection path 和 validation failure facts 复制到新的 `FieldContract`。分层方向保留，但复制字段模型的实现选择由 Decision 9 取代。

### Decision 3: 将 fixed source slots 改为 ordered source collection

现有 `DirectInput / ProjectConfig / UserConfig / Default` 模型泛化为 ordered sources。Priority 数值越大优先级越高；同 priority 时，后注册 source 的 candidate 获胜。`Append` 与 `MapMerge` 从低 priority 到高 priority 应用，同 priority 内按 source 注册顺序应用；因此后应用的 map value 覆盖同名 key。Resolver 执行这套固定顺序，不硬编码 Docnav 的四层命名。

### Decision 4: Resolution 返回最终值和 provenance trace

Resolver 返回 resolved values、diagnostics 和 provenance facts。Trace 至少保留 selected source、overridden 或 merge contributors、default fallback 和 source locator；readable explain 从 trace 派生，不从最终 struct 反推。

### Decision 5: Merge strategy 属于 field-level resolution policy（由 Decision 9 收敛）

第一轮把 merge policy 作为 resolution-owned、按 field identity 关联的声明。Field-level 方向保留，但 metadata owner 和 public surface 已由 Decision 9 收敛：声明直接进入 canonical `FieldDef`，resolver 只负责执行。

### Decision 6: Framework integrations 保持 companion crate 层（由 Decision 12 补充）

Clap arg 生成/读取进入 `cli-config-resolution-clap`；structured config 抽取进入 `cli-config-resolution-serde`。Core 不依赖单一 CLI/config framework，derive convenience 不在本轮新增。

### Decision 7: Docnav 集成通过 hard cutover 一步切换

Docnav 保持 hard cutover。实现完成状态不保留旧 resolver 运行路径、runtime feature flag 或 fallback 开关；回滚方式是代码级 revert。

### Decision 8: 审计出口与 release-readiness 决策门

工作区 package 名继续使用 `cli-config-resolution`、`cli-config-resolution-clap` 和 `cli-config-resolution-serde`。Canonical public repository 已确认为 `https://github.com/zxyycom/cli-config-resolution`；外部发布前仍需重新确认 package 名。许可证选择、version 和发布顺序继续作为独立 release decisions，不阻塞本 change 建立并验证独立 Cargo workspace 子仓库。

### Decision 9: `FieldDef` / `FieldDefSet` 是唯一 canonical 参数模型

`docnav-typed-fields::FieldDef` 与 `FieldDefSet` 直接承载标准参数的 identity、类型、约束、默认值、`MergeStrategy`、校验和 typed materialization。`MergeStrategy` 的 public surface 固定为 `Replace`、`Append`、`MapMerge`、`DenyConflict`：`Replace` 同时适用于 scalar、list 和 map，并是未显式声明时的默认值。`cli-config-resolution` 依赖并 re-export canonical types，允许提供 `Parameter` / `ParameterSet` 薄别名或 convenience constructors，但不得复制状态或建立需要同步的第二套 field/value/constraint/default/validation/merge model。

为 resolution 暴露 metadata 时，把现有 `SchemaMetadataView` / `ProcessingMetadataView`、field merge strategy 和必要 accessor 调整为稳定 public API。Resolver 只读取 `FieldDef` 上的 merge metadata 并执行，不拥有独立声明入口，也不按 field identity 维护第二份 policy table。

### Decision 10: 复用 processing 机制声明显式 extraction strategy

`ProcessStrategy` 继续作为字段的 extraction metadata owner，并整理出显式的 CLI flag、env var 和 config path constructors/variants。每个 strategy 记录 source-local locator，并通过 existing `ProcessingId` 选择一组抽取规则；config path 可以复用现有 JSON path 表达，不新增平行 projection graph。

Extractor 只遍历 `FieldDefSet` 对相应 processing/source 声明的 metadata，将存在或非法的输入映射为统一 candidate，并保留 field identity、source id/kind 和 locator。输入缺失可以由“没有 candidate”表达；只有 trace 或错误契约确实需要时才保留额外状态。

### Decision 11: Resolution 协调 canonical validation、merge 与 materialization

Resolution core 拥有 ordered source collection、priority、merge execution、diagnostic facts 和 provenance。CLI/env 字符串及 structured config value 保留其 decode 成功或失败事实；resolver 根据 `FieldDef` 声明的策略确定 selected、contributors 和 overridden candidates。Selected 或 contributing candidate 只要解码失败就阻断该 field resolution；被 `Replace` 覆盖的非法 candidate 只进入 trace，不阻断更高优先级的有效 selected candidate。合并完成后，resolver 必须使用同一 canonical `FieldDef` validation metadata 再校验最终值。静态默认值从 `FieldDef` metadata 生成最低优先级 fallback，不要求消费者手工声明 default source candidate。

Typed materialization 继续复用 `FieldDefSet` / derive 支持。Resolution 只提供完成 materialization 所需的 canonical field values 和 trace，不再实现一套独立 typed-output contract。

### Decision 12: Source-specific adapter 与未知输入策略保持简单

`cli-config-resolution-clap` 根据 CLI extraction strategy 注册或读取 clap arguments；未注册 flag 的拒绝沿用 clap 原生行为。Env extractor 接收 `IntoIterator<Item = (String, String)>` 或等价可测试输入，只查询声明过的 env name。Config companion 首批以当前 `serde_json::Value` structured document 为事实边界，只查询声明过的 config path。

Env/config 中未声明的 key 默认静默忽略。当前 public API 不增加 `UnknownPolicy`、全量 key 扫描或 unused-key diagnostics；以后有明确消费者需求时再通过独立 change 增加 strict mode。

### Decision 13: 独立子仓库是 Cargo workspace

目标子仓库不是单一 core package，而是一个可独立 checkout、build 和 test 的 Cargo workspace。它可以包含 typed-fields、typed-fields macros、resolution core、clap companion 和 serde/config companion；workspace root 统一 version、edition、repository 和 shared dependencies。当前未选择 license，package metadata 有意不声明 license；license selection 延后到 release decision。`cli-config-resolution` 是主要消费者入口，re-export canonical 参数类型，companion crates 依赖该入口。

该 workspace 不依赖 Docnav protocol、adapter contracts、navigation、output 或 Markdown adapter crates。Docnav-specific mapping 和 hard-cutover tests 保留在 Docnav repository。Docnav 通过 `subrepos/cli-config-resolution` Git submodule 接入公开仓库，并由父仓库 gitlink 固定已验证 revision；首次 clone 运行 `git submodule update --init --recursive`。crates.io 发布和 license selection 仍是单独的 release decisions。

### Decision 14: Docnav 集成使用 canonical metadata，不再复制字段声明

Docnav 的 existing `FieldDefSet` 直接传给 extractor/resolver。Explicit input、project config、user config、built-in defaults 和 selected adapter native options 映射到通用 source model；adapter applicability、handler binding、request construction 和 diagnostic-code mapping 继续由 Docnav 拥有。

第一轮 `generic_field_set`、手工字段 metadata 转换和第二套 resolver 只能作为迁移输入，不属于完成后的 runtime path。完成状态以 Docnav 使用与独立 workspace 示例相同的 canonical `Source` / `SourceCandidate` / `Resolver` public contract 为准；已解析或 custom input 的 consumer-owned adapter 由 Decision 15 约束。

### Decision 15: 已解析与 custom input 使用 consumer-owned source adapter

当 consumer 在进入 resolution 前已经拥有解析后的 application input 时，不强制重新构造 framework object 或使用不匹配的 companion。Consumer 可以实现私有 source adapter，但该 adapter 必须只遍历 canonical `FieldDefSet` 的 processing metadata，把已存在的输入映射为公共 `Source` / `SourceCandidate`；它不得维护平行 locator table，也不得复制 field identity、value kind、constraint、default、validation 或 merge metadata。

存在原始 clap matches、environment key/value iterator 或 structured config document 时，仍优先使用对应 public extractor。Docnav 的 direct/native CLI adapter 保留在 `docnav-navigation` 内，只拥有 already-parsed input 到 candidate 的 application mapping，不进入通用 workspace，也不改变 source priority、adapter applicability、diagnostic mapping 或 request construction owner。

## Risks / Trade-offs

- [Risk] typed-fields public metadata 目前偏向 crate-internal 使用 → Mitigation: 只公开 resolver/extractor 必需的 immutable views 和 validation entry points，用 consumer compile tests 固定最小 surface。
- [Risk] 把 source locator 直接塞进 `ProcessStrategy` 可能让 typed-fields 感知来源名 → Mitigation: strategy 只描述 extraction locator，不拥有 source priority、merge、framework object 或 application behavior；这些继续由 resolution/companion 拥有。
- [Risk] 非法低优先级输入是否阻断存在歧义 → Mitigation: `Replace` 只阻断 selected candidate 的 decode failure，被覆盖的非法 candidate 仅进入 trace；`Append`、`MapMerge` 和 `DenyConflict` 所需 contributors 任一 decode failure 都阻断，最终 merged value 始终再经 canonical validation。
- [Risk] 子仓库化同时移动 typed-fields 与 companions 会扩大改动面 → Mitigation: 以 Cargo workspace 为原子边界，先通过独立 checkout tests，再切换 Docnav dependency source；不保留第二套兼容 wrapper。
- [Risk] hard cutover 放大参数解析回归影响 → Mitigation: 以 CLI + env + config + default 的端到端示例和 Docnav 等价测试作为删除转换层的前置门。
- [Risk] consumer-owned source adapter 可能重新引入 locator 或字段事实复制 → Mitigation: adapter 只接受 canonical processing metadata 与已解析 input，测试固定 canonical `ProcessStrategy` authoring，并用残留审计拒绝平行 source declaration state。

## Implementation Plan

1. 公开 `FieldDef` / `FieldDefSet` resolution 所需的最小 metadata 和 validation API，并由 `cli-config-resolution` re-export canonical 类型。
2. 删除或内部化 duplicate `FieldContract` / `FieldSet` 及平行 value/constraint/default/validation/merge model；把默认 `Replace` 的四项 `MergeStrategy` public surface 直接加入 canonical `FieldDef` metadata。
3. 在 existing processing contract 上增加 CLI/env/config 的显式 extraction strategy，并让 clap、env、serde/config extractor 产出统一 candidates。
4. 整理 resolver，固定 higher-priority/later-registration winner 与 low-to-high merge order，并按 selected/contributing/overridden 规则处理 decode failure；最终 merged value 复用 canonical validation 与 typed-fields materialization。
5. 让 Docnav 删除 `generic_field_set` 与手工字段 metadata 转换；framework inputs 使用 public extractor，已解析/custom inputs 使用 Decision 15 限定的私有 source adapter，再进入同一 public resolver path。
6. 把 typed-fields/macros、resolution core 和 companions 组织为独立 Cargo workspace 子仓库，并验证独立 checkout 与 Docnav consumer。
7. 更新 example、README、tests 和验证记录；所有新任务完成后再恢复 change 的完成状态。

## Open Questions

无。Canonical 字段模型与 merge metadata owner、priority/tie/merge ordering、validation timing、抽取策略 owner、未知输入默认行为、Docnav hard cutover、consumer-owned source adapter 和独立 Cargo workspace 边界均已由 Decisions 3、9-15 收敛。
