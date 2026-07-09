本 design 定义 `replace-config-commands-with-inspect` 的实现前决策：用单一只读 `docnav config inspect` 替换旧 config 子命令，把现有 CLI flag owner-map 模式泛化为可同时服务 CLI/input 与 config-source 的 owner-provided parameter aggregation metadata，并把 adapter native option 持久配置路径迁移到 `options.<adapter-id>.<option-key>`。

## 文档重心

本 design 只承接会影响实现形状的决策、替代方案、风险和迁移计划。它不重复所有 spec scenarios，也不作为任务执行 checklist。若正文需要写“必须如何验收”，应落到 `specs/*/spec.md`；若需要写“先做哪一步”，应落到 `tasks.md`。

## Context

Docnav 当前有两条配置校验路径。Navigation input resolution 会把通用字段和 selected adapter `AdapterOptionSpec` 注册进同一个 `FieldDefSet`，再通过 typed-field helper 做 typed extraction、constraint validation 和 source-attributed diagnostic handoff。Core config commands 则在 `docnav` crate 中用 `CoreConfig` serde model、手写 shape check、手写 `set_key` match 分支和 adapter registry key lookup 完成读取与写入校验。

这导致 config command 和 navigation resolution 之间存在事实源漂移：core 可以把未按 adapter option value kind/range 校验的裸 `options.<key>` 写入 config file；navigation 后续读取同一值时才通过 selected adapter typed-field declaration 报错。裸 `options.<key>` 还不能稳定区分不同 adapter 的同名 native option。长期看，core config 命令还会继续复制 output enum、positive integer、outline mode、adapter option declaration、nested config shape 和 diagnostic mapping 等字段事实。

本 change 不改变 raw protocol、readable output wrapper 或 adapter handler payload。它改变的是现有输入 owner map 如何抽象为参数汇总边界，config source validation 如何找到字段事实源，`docnav config` 如何从可变编辑器收缩为只读检查命令，以及 adapter-owned native options 如何在 config path 中通过 adapter id 定位 owner。

## Goals / Non-Goals

**Goals:**

- 将现有 CLI flag owner-map 模式抽象为参数汇总边界，让 CLI/input projection 与 config-source projection 使用同一份 owner-provided parameter metadata 表达字段级类型、约束、默认值、processing path 和 source binding facts；实际 source attribution 仍由 navigation/config consumer 处理。
- 保留 owner 边界：core 拥有 config CLI surface 和 core-owned `invocation_log.*`；navigation 拥有 config source loading、source priority、selected adapter field set 和 request construction；adapter 拥有 native option semantics；typed-fields 只提供字段事实和投影 helper。
- 将 `docnav config` 收缩为一个只读 inspect command，展示配置来源、来源状态和可验证的 source facts。
- 将 adapter native option 的 persistent config path 固定为 `options.<adapter-id>.<option-key>`，避免把同名 adapter option key 折叠成 core-owned 全局字段。
- 先审计并复用现有 owner-specific 数组配置校验；只有当 source inspection、direct config read 与 navigation resolution 需要同一 typed-field projection 才能稳定覆盖当前数组配置时，才扩展 typed-fields 的 processing-path helper。
- 保持已有 document operation protocol/readable 输出兼容；配置校验错误继续通过现有 diagnostic/output 机制投影。
- 用测试和验证材料证明 config inspect/read validation 与 navigation read/resolution validation 使用同一 metadata 来源。

**Non-Goals:**

- 不改变 `RequestEnvelope`、`ProtocolResponse`、readable-view 或 readable-json payload shape。
- 不改变 linked adapter handler 的输入边界；handler 继续接收 typed arguments，不接收 raw config JSON。
- 不把 `options.<adapter-id>.*` 改成 core-owned namespace；同名 native option 仍可由不同 adapter 在各自 adapter id namespace 下声明。
- 不保留 `config set`、`config unset`、`config get` 或 `config list` 作为长期命令；这是破坏式 CLI surface 收缩。
- 不兼容旧的裸 `options.<key>`；旧路径不做迁移、不做兼容读取、不做特殊提示。
- 不新增 config editor、JSON patch、数组编辑 DSL 或 CLI token decoding 规则。
- `config inspect` 的首版 scope 是 source inspection；它可以列出当前输入可解析出的参数事实，但不预演 selected adapter/operation dispatch。
- 不要求 typed-fields 拥有 config source priority、CLI flags、CLI lexical token decoding、adapter selection、protocol envelope 或 public diagnostic code。

## Decisions

### Decision 1: 泛化现有 owner map 为参数汇总 projection

实现必须把现有 CLI flag owner-map 模式泛化为参数汇总 projection。各 owner 继续提供自己的字段声明；参数汇总层负责从同一批 owner-provided metadata 产出 CLI/input projection 和 config-source projection：

- core CLI 提供 core-owned runtime config 字段，例如 `invocation_log.*`。
- navigation 提供 navigation-owned config 字段，例如 `defaults.adapter`、`defaults.pagination.*`、`defaults.output`、`outline.*`。
- navigation aggregation 将 registered adapter 的 `AdapterOptionSpec` typed-field declarations 投影到 adapter id namespace，例如 `options.markdown.max_heading_level`。
- typed-fields 提供按 processing id/path 查找、validate value、返回 canonical typed value 和列举可接受字段 metadata 的 helper；compound structure helper 只在当前数组配置无法由既有 owner-specific 校验稳定覆盖时扩展。

Core config inspect 与 navigation input resolution 都消费参数汇总层产出的 config-source projection；两者不各自重新维护字段事实。CLI flag 解析继续消费 CLI/input projection。

Alternative considered: 继续在 core config commands 中维护 `SUPPORTED_CORE_KEYS`、shape switch 和 per-key parse 函数。该方案短期最小，但继续复制字段事实，无法解决 adapter option value validation drift。

### Decision 2: `docnav config` 收缩为单一只读 inspect command

长期 CLI surface 只保留一个配置命令：`docnav config inspect`。该命令不修改文件，不接受 key/value 写入，不实现 unset，也不保留单 key get。它负责读取 selected project/user config paths，展示每个来源的 path、origin、存在性、load state、JSON/config validation issue 和可解析 source summary。

`docnav config set`、`docnav config unset`、`docnav config get` 和 `docnav config list` 在本 change 中作为破坏式迁移被移除，不提供兼容 alias。配置修改由用户直接编辑 JSON config file 完成；Docnav 的责任是初始化模板、读取、校验、解释和检查配置来源。

Alternative considered: 保留 `get/list` 并只移除 `set/unset`。该方案仍会维护多套 config CLI 输出，并继续诱导用户把 config command 当成编辑入口；单一 inspect command 更符合“配置文件直接编辑”的边界。

### Decision 3: Adapter native options 使用 `options.<adapter-id>.<option-key>`

Adapter native option value validation 必须来自路径中 adapter id 指向的 adapter declaration。Canonical persistent config path 为 `options.<adapter-id>.<option-key>`，例如 `options.markdown.max_heading_level`。Adapter declaration 提供 option facts；registry/navigation aggregation 提供 adapter id segment 和 config path projection。Adapter id 直接使用现有 adapter registry id，不新增 alias、display id 或兼容映射。

这是本 change 的明确迁移子范围。它会要求同步 owner docs、schema、examples、tests 和少量 hard-coded config path / typed-field path 逻辑，但不改变 adapter handler payload、`OperationArguments` 的 adapter-owned typed handoff 语义或 raw/readable protocol。实现上应优先替换现有 `options.<key>` path parser、registry lookup 和 key registry，而不是新增一套 adapter option system。

同名 option key 可以在不同 adapter id namespace 下共存，shared layer 不做跨 adapter compatibility 判断。若同一个 adapter 在同一个 config path 下为多个 operation 声明了不兼容 metadata，必须报告 adapter-local declaration conflict，或者要求 adapter 暴露不同 config key；core 不得根据 operation 猜测其中一个 declaration。裸 `options.<option-key>` 只是 unknown/invalid config path，不做特殊迁移或兼容读取。

Alternative considered: 通过 document context 解释裸 `options.<key>`。该方案把持久配置形态绑定到某个文档选择流程，仍会让同名 key 的含义不稳定；adapter-id namespace 更直接，也更适合作为持久配置格式。

### Decision 4: Config store 读取校验分为 shape、metadata 和 owner-specific policy

Config file 读取必须先保留 JSON source load diagnostics，再使用参数汇总产出的 config-source projection 检查 unknown field、declared `options.<adapter-id>.*` source 和可由 projection 表达的 typed value failures。字段 value validation 应尽量通过 typed-field metadata；expected object/array shape 与 nested value failures 先复用现有 owner-specific validation。只有当现有 owner-specific path 无法同时满足 source inspection、direct config read 与 navigation resolution 的一致性时，才把对应结构迁移到 typed-fields helper。

`outline.mode_rules[]`、`outline.auto_full_read.thresholds[]` 等非结构化全文读取相关数组配置是本 change 的审计目标，不是 typed-fields 的先验扩展目标。实现必须先确认既有 `outline` owner validation、config key registry 和参数汇总 projection 是否足够表达这些数组字段的 source path、unknown item key、required member 和 typed value diagnostics。若足够，则保留 owner-specific validation 并通过 tests 固定 parity；若不足，则只为这些当前字段增加最小 typed-fields compound metadata helper，表达 array item schema、object members 和 nested failure path，不实现通用 JSON schema engine。

Alternative considered: 只把旧 config 命令接入 typed-fields，保持 config store 读取手写 shape validation。该方案仍会让直接编辑配置文件与运行时配置读取的诊断和字段接受范围漂移。

### Decision 5: Config inspect 聚焦 source inspection

`docnav config inspect` 展示 selected config sources、source summary、load state、config validation diagnostics，以及当前输入可解析出的参数事实。Adapter-id namespaced values 在 inspect 中作为具体配置来源字段呈现，并通过 owner-provided metadata 校验；selected adapter/operation 的有效参数构造继续归 navigation input resolution。Inspect 输出不得声称某个 adapter option 已被 dispatch，也不得替代 navigation 的 selected-adapter validation。

### Decision 6: Diagnostic mapping 保持 owner-aware

Unsupported key、unknown adapter id、unknown config field、invalid config object/array、invalid value、adapter-local declaration conflict 和 selected adapter native option invalid 必须映射到现有 diagnostics/output 边界。Diagnostic details 必须携带 config source level/path、field/key、owner/adapter id when applicable、nested path when applicable，以及 typed-field validation reason。

Machine-readable error shape 必须保持由 diagnostics/protocol/output owner 投影；本 change 只定义 config validation 如何产生 owner-aware diagnostic facts。

### Decision 7: 迁移按 docs-first vertical slice 进行

迁移顺序必须先让 owner 主规范闭合，再按可观察 vertical slice 建立证明目标、补测试并实现。每个 slice 先确认 owner 文档中的可观察语义，再补最小失败/契约测试，然后实现对应行为。

推荐 slice 顺序：先收缩 CLI 到只读 `config inspect` 并展示 source status，再接入参数汇总的 scalar metadata 与 adapter-id option path，然后审计非结构化全文读取相关数组配置是否需要 typed-fields 最小扩展。不得新增 CLI token decoding、JSON patch 或 write canonicalization 逻辑来替代被移除的写入命令。

## Risks / Trade-offs

- [Risk] 移除 `config set|get|unset|list` 是 breaking change。Mitigation: 本 change 明确接受破坏式迁移；docs/tests/help 必须同步删除旧命令，只保留 `config inspect`。
- [Risk] `options.<adapter-id>.<option-key>` 会改变裸 `options.<key>` 的配置形态，并触及较多文档和验证材料。Mitigation: canonical path 固定为 adapter-id namespace；旧裸路径按普通 unknown/invalid config 处理；实现重点放在现有 hard-coded path、registry lookup、schema/example/test 同步，不扩大 adapter handler 或 protocol scope。
- [Risk] typed-fields projection 被迫承担 consumer policy。Mitigation: typed-fields 只提供 field facts、processing path lookup、candidate JSON value validation 和 canonical typed value；source priority、CLI flags、adapter selection 和 diagnostic code 仍由 consumer owner 决定。
- [Risk] typed-fields compound validation 范围扩大，可能让实现触及通用 JSON schema engine。Mitigation: 先证明现有 owner-specific 数组配置校验是否足够；只有不足时才实现当前 config-source 校验所需的最小 compound subset，并用 typed-fields 单元测试锁定 scalar 与 compound path parity；不把 source priority、CLI decoding 或通用 schema policy 放进 typed-fields。
- [Risk] 同一个 adapter 对同一 config key 声明 operation-specific 不兼容 metadata。Mitigation: 作为 adapter-local declaration conflict 处理，要求 adapter 使用不同 config key 或不把该 option 暴露为 persistent config path。
- [Risk] source inspection 和 operation argument construction 容易被混成一个输出目标。Mitigation: inspect 输出配置来源事实；navigation input resolution 负责 selected adapter/operation 的有效参数构造。

## Complexity Assessment

- 单一 `config inspect` source status 难度较低：现有 selected project/user path、origin 和 config source load diagnostics 已经存在，主要是 CLI surface、输出形状和 tests 的收缩。
- Adapter-id option namespace 难度较低到中等：主要改 contract、config path parser、adapter registry lookup、hard-coded typed-field path、docs/schema/examples/tests；它触及范围大，但不改变 adapter handler 或 protocol，工程风险可控。
- typed-fields compound validation 难度取决于审计结果：如果现有 owner-specific outline validation 能覆盖 source inspection/direct read/navigation parity，则只需测试和边界记录；如果不能，需要为 `FieldDefSet` 增加当前 config 所需的 array/object/nested path metadata、JSON candidate traversal 和 nested failure reporting。范围限定为当前配置校验需要的 subset；不做通用 schema engine。
- 推荐实现顺序：owner 主规范闭合后，先收缩 CLI 到只读 inspect 并输出 source status，再完成参数汇总的 scalar config-source projection 与 adapter-id option path，然后评估并按需实现 typed-fields 最小 compound helper。

## Migration Plan

1. 更新 owner 主规范，固定参数汇总 projection、单一 `config inspect` surface、`options.<adapter-id>.<option-key>` canonical path、diagnostic mapping 和 direct config file read parity。
2. 建立实现前证明目标和最小失败/契约测试，覆盖旧 config 子命令移除、source status、参数汇总 parity、adapter-id namespace，以及非结构化全文读取相关数组配置的现有校验 parity。
3. 迁移 core config command/store：删除 `set|get|unset|list`，实现只读 inspect source status，并列出当前输入可解析出的参数事实。
4. 在参数汇总层接入 scalar config-source projection 与 adapter-id namespaced declarations，让 config inspect/source validation/navigation resolution 消费同一 projection。
5. 审计 `outline.mode_rules[]`、`outline.auto_full_read.thresholds[]` 等数组配置是否可继续由 owner-specific validation 与参数汇总 projection 覆盖；只有不足时，才为 typed-fields 增加或暴露当前 config-source subset 所需的 compound metadata、candidate validation 和 nested failure helper。
6. 补充 docs/schema/example 验证材料，覆盖 config inspect、config read、navigation resolution parity、adapter-id native option namespace、nested config shape 和 diagnostics。
7. 运行每个 slice 的局部 Rust/CLI 验证、OpenSpec validation；跨 crate 或 docs/schema/example 同步完成后运行 workspace verifier。

Rollback 策略：由于 change 不改变 protocol/readable output 和 adapter handler payload，若迁移引入回归，可以临时保留旧内部 config loading helper，但不得恢复 `set|get|unset|list` 作为 public surface，也不得保留会绕过 typed-field metadata 的新 public behavior。

## Open Questions

已收敛：`docnav config` 长期只保留只读 `inspect`；现有 CLI flag owner-map 模式要泛化为参数汇总；`options.<adapter-id>.<option-key>` 使用现有 adapter registry id，并作为本 change 的明确配置路径迁移子范围推进；旧裸 `options.<key>` 不兼容不迁移；inspect scope 聚焦 source inspection，并列出当前输入可解析出的参数事实，不预演 dispatch；typed-fields 不先验扩展，先确认非结构化全文读取相关数组配置是否能由现有 owner-specific validation 与参数投影覆盖，不能覆盖时才实现当前所需最小 subset。无未回答开放问题；剩余工作是实现阶段审计、验证和必要的 owner-specific policy 收窄，不作为设计问题保留。
