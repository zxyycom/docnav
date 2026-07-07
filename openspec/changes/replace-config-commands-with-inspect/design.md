本 design 定义 `replace-config-commands-with-inspect` 的实现前决策：用单一只读 `docnav config inspect` 替换旧 config 子命令，并让配置读取校验与配置检查复用 owner-provided config metadata。

当前 change 只在 `openspec/changes/replace-config-commands-with-inspect/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## Context

Docnav 当前有两条配置校验路径。Navigation input resolution 会把通用字段和 selected adapter `AdapterOptionSpec` 注册进同一个 `FieldDefSet`，再通过 typed-field helper 做 typed extraction、constraint validation 和 source-attributed diagnostic handoff。Core config commands 则在 `docnav` crate 中用 `CoreConfig` serde model、手写 shape check、手写 `set_key` match 分支和 adapter registry key lookup 完成读取与写入校验。

这导致 config command 和 navigation resolution 之间存在事实源漂移：core 可以把未按 adapter option value kind/range 校验的裸 `options.<key>` 写入 config file；navigation 后续读取同一值时才通过 selected adapter typed-field declaration 报错。裸 `options.<key>` 还不能稳定区分不同 adapter 的同名 native option。长期看，core config 命令还会继续复制 output enum、positive integer、outline mode、adapter option declaration、nested config shape 和 diagnostic mapping 等字段事实。

本 change 不改变 raw protocol、readable output wrapper 或 adapter handler payload。它改变的是 config source validation 如何找到字段事实源，`docnav config` 如何从可变编辑器收缩为只读检查命令，以及 adapter-owned native options 如何在 config path 中通过 adapter id 定位 owner。

## Goals / Non-Goals

**Goals:**

- 让配置读取、只读配置检查和 navigation input resolution 使用同一份 owner-provided config metadata 表达字段级类型、约束、默认值、processing path 和 source binding facts；实际 source attribution 仍由 navigation/config consumer 处理。
- 保留 owner 边界：core 拥有 config CLI surface 和 core-owned `invocation_log.*`；navigation 拥有 config source loading、source priority、selected adapter field set 和 request construction；adapter 拥有 native option semantics；typed-fields 只提供字段事实和投影 helper。
- 将 `docnav config` 收缩为一个只读 inspect command，展示配置来源、来源状态和可验证的 source facts。
- 将 adapter native option 的 persistent config path 固定为 `options.<adapter-id>.<option-key>`，避免把同名 adapter option key 折叠成 core-owned 全局字段。
- 扩展 typed-fields 的 config metadata projection，使它能声明和校验 array、object、nested structure 的 config source shape，并在 nested path 上返回 typed validation failure。
- 保持已有 document operation protocol/readable 输出兼容；配置校验错误继续通过现有 diagnostic/output 机制投影。
- 用测试和验证材料证明 config inspect/read validation 与 navigation read/resolution validation 使用同一 metadata 来源。

**Non-Goals:**

- 不改变 `RequestEnvelope`、`ProtocolResponse`、readable-view 或 readable-json payload shape。
- 不改变 linked adapter handler 的输入边界；handler 继续接收 typed arguments，不接收 raw config JSON。
- 不把 `options.<adapter-id>.*` 改成 core-owned namespace；同名 native option 仍可由不同 adapter 在各自 adapter id namespace 下声明。
- 不保留 `config set`、`config unset`、`config get` 或 `config list` 作为长期命令；这是破坏式 CLI surface 收缩。
- 不兼容旧的裸 `options.<key>`；旧路径不做迁移、不做兼容读取、不做特殊提示。
- 不新增 config editor、JSON patch、数组编辑 DSL 或 CLI token decoding 规则。
- `config inspect` 的首版 scope 是 source inspection；adapter-id namespace 的有效值由 config source validation 和 navigation input resolution 消费。
- 不要求 typed-fields 拥有 config source priority、CLI flags、CLI lexical token decoding、adapter selection、protocol envelope 或 public diagnostic code。

## Decisions

### Decision 1: 建立 owner-supplied config metadata projection

实现必须提供一个面向 config source 的 metadata projection，它由各 owner 提供字段声明并聚合成可查询结构：

- core CLI 提供 core-owned runtime config 字段，例如 `invocation_log.*`。
- navigation 提供 navigation-owned config 字段，例如 `defaults.adapter`、`defaults.pagination.*`、`defaults.output`、`outline.*`。
- navigation aggregation 将 registered adapter 的 `AdapterOptionSpec` typed-field declarations 投影到 adapter id namespace，例如 `options.markdown.max_heading_level`。
- typed-fields 提供按 config processing path 查找、validate value、validate structure、返回 canonical typed value 和列举可接受字段 metadata 的 helper。

Alternative considered: 继续在 core config commands 中维护 `SUPPORTED_CORE_KEYS`、shape switch 和 per-key parse 函数。该方案短期最小，但继续复制字段事实，无法解决 adapter option value validation drift。

### Decision 2: `docnav config` 收缩为单一只读 inspect command

长期 CLI surface 只保留一个配置命令：`docnav config inspect`。该命令不修改文件，不接受 key/value 写入，不实现 unset，也不保留单 key get。它负责读取 selected project/user config paths，展示每个来源的 path、origin、存在性、load state、JSON/config validation issue 和可解析 source summary。

`docnav config set`、`docnav config unset`、`docnav config get` 和 `docnav config list` 在本 change 中作为破坏式迁移被移除，不提供兼容 alias。配置修改由用户直接编辑 JSON config file 完成；Docnav 的责任是初始化模板、读取、校验、解释和检查配置来源。

Alternative considered: 保留 `get/list` 并只移除 `set/unset`。该方案仍会维护多套 config CLI 输出，并继续诱导用户把 config command 当成编辑入口；单一 inspect command 更符合“配置文件直接编辑”的边界。

### Decision 3: Adapter native options 使用 `options.<adapter-id>.<option-key>`

Adapter native option value validation 必须来自路径中 adapter id 指向的 adapter declaration。Canonical persistent config path 为 `options.<adapter-id>.<option-key>`，例如 `options.markdown.max_heading_level`。Adapter declaration 提供 option facts；registry/navigation aggregation 提供 adapter id segment 和 config path projection。Adapter id 直接使用现有 adapter registry id，不新增 alias、display id 或兼容映射。

同名 option key 可以在不同 adapter id namespace 下共存，shared layer 不做跨 adapter compatibility 判断。若同一个 adapter 在同一个 config path 下为多个 operation 声明了不兼容 metadata，必须报告 adapter-local declaration conflict，或者要求 adapter 暴露不同 config key；core 不得根据 operation 猜测其中一个 declaration。裸 `options.<option-key>` 只是 unknown/invalid config path，不做特殊迁移或兼容读取。

Alternative considered: 通过 document context 解释裸 `options.<key>`。该方案把持久配置形态绑定到某个文档选择流程，仍会让同名 key 的含义不稳定；adapter-id namespace 更直接，也更适合作为持久配置格式。

### Decision 4: Config store 读取校验分为 shape、metadata 和 owner-specific policy

Config file 读取必须先保留 JSON source load diagnostics，再使用 config metadata 检查 unknown field、expected object/array shape、declared `options.<adapter-id>.*` source 和 nested value failures。字段 value validation 与可表达的 structure validation 应尽量通过 typed-field metadata；无法由 typed-fields 表达的 owner-specific policy 必须保持在 owner 层，并在 design/tasks 中标注迁移条件。

`outline.mode_rules[]`、`outline.auto_full_read.thresholds[]` 等数组项字段必须由 typed-fields 的 compound metadata 表达可接受 item fields、required/optional object members、array item schema 和 nested failure path。实现方向是扩展 typed-fields 支持 array、object、nested structure validation，而不是继续扩大 config store 的手写 shape registry。若实现审计发现短期无法一次覆盖所有 compound pattern，只能保留有测试、TODO、范围和移除条件的窄 owner policy。

Alternative considered: 只把旧 config 命令接入 typed-fields，保持 config store 读取手写 shape validation。该方案仍会让直接编辑配置文件与运行时配置读取的诊断和字段接受范围漂移。

### Decision 5: Config inspect 聚焦 source inspection

`docnav config inspect` 展示 selected config sources、source summary、load state 和 config validation diagnostics。Adapter-id namespaced values 在 inspect 中作为具体配置来源字段呈现，并通过 owner-provided metadata 校验；selected adapter/operation 的有效参数构造继续归 navigation input resolution。

### Decision 6: Diagnostic mapping 保持 owner-aware

Unsupported key、unknown adapter id、unknown config field、invalid config object/array、invalid value、adapter-local declaration conflict 和 selected adapter native option invalid 必须映射到现有 diagnostics/output 边界。Diagnostic details 必须携带 config source level/path、field/key、owner/adapter id when applicable、nested path when applicable，以及 typed-field validation reason。

Machine-readable error shape 必须保持由 diagnostics/protocol/output owner 投影；本 change 只定义 config validation 如何产生 owner-aware diagnostic facts。

### Decision 7: 迁移以 tests-first parity 进行

实现必须先用当前失败点建立 tests：旧 `config set|get|unset|list` 不再是 accepted CLI；`config inspect` 能展示 project/user source 状态；同一 config value 在 config source validation 与 navigation resolution 中应使用同一 metadata 得到一致的成功或失败；同名 native option 在不同 adapter id namespace 下必须 deterministic；config file direct edit 的 unknown/invalid nested field diagnostics 仍保留 source path。

之后再迁移 config store/navigation source validation 到 shared config metadata projection，并删除或收窄手写 shape registry。不得新增 CLI token decoding、JSON patch 或 write canonicalization 逻辑来替代被移除的写入命令。

## Risks / Trade-offs

- [Risk] 移除 `config set|get|unset|list` 是 breaking change。Mitigation: 本 change 明确接受破坏式迁移；docs/tests/help 必须同步删除旧命令，只保留 `config inspect`。
- [Risk] `options.<adapter-id>.<option-key>` 会改变裸 `options.<key>` 的配置形态。Mitigation: canonical path 固定为 adapter-id namespace；旧裸路径按普通 unknown/invalid config 处理。
- [Risk] typed-fields projection 被迫承担 consumer policy。Mitigation: typed-fields 只提供 field facts、processing path lookup、candidate JSON value validation 和 canonical typed value；source priority、CLI flags、adapter selection 和 diagnostic code 仍由 consumer owner 决定。
- [Risk] typed-fields compound validation 范围扩大，可能让实现触及 array/object traversal、nested diagnostics 和 canonical JSON serialization。Mitigation: 先实现 config-source 所需的 JSON compound schema subset，并用 typed-fields 单元测试锁定 scalar 与 compound path parity；不把 source priority 或 CLI decoding 放进 typed-fields。
- [Risk] 同一个 adapter 对同一 config key 声明 operation-specific 不兼容 metadata。Mitigation: 作为 adapter-local declaration conflict 处理，要求 adapter 使用不同 config key 或不把该 option 暴露为 persistent config path。
- [Risk] source inspection 和 operation argument construction 容易被混成一个输出目标。Mitigation: inspect 输出配置来源事实；navigation input resolution 负责 selected adapter/operation 的有效参数构造。

## Complexity Assessment

- 单一 `config inspect` source status 难度较低：现有 selected project/user path、origin 和 config source load diagnostics 已经存在，主要是 CLI surface、输出形状和 tests 的收缩。
- Adapter-id option namespace 难度较低到中等：主要改 contract、config path parser、adapter registry lookup、docs/tests；它删除了跨 adapter compatibility 判断和 selected context surface，降低长期复杂度。
- typed-fields compound validation 难度中高：需要为 `FieldDefSet` 增加 array/object/nested path metadata、JSON candidate traversal、nested failure reporting 和 canonical serialization。若只覆盖当前 config 所需的 object/array subset，预计可以分阶段完成；若一次做成通用 schema engine，范围会明显扩大，不建议在本 change 中这么做。
- 推荐实现顺序：先收缩 CLI 到只读 inspect 并输出 source status，再完成 scalar config metadata 与 adapter-id option path，然后扩展 typed-fields compound subset 覆盖 `outline.*` 现有结构。

## Migration Plan

1. 更新 owner 主规范，固定 config metadata projection、单一 `config inspect` surface、`options.<adapter-id>.<option-key>` canonical path、diagnostic mapping 和 direct config file read parity。
2. 为 typed-fields 增加或暴露按 processing path 查询 metadata、validate JSON candidate、validate compound config shape、输出 canonical typed value、列举 config path 的 helper。
3. 在 navigation/adapter-contract 层建立 config metadata aggregation，复用 common fields 和 adapter-id namespaced declarations。
4. 迁移 core config command/store：删除 `set|get|unset|list`，实现只读 inspect source status，先用新 metadata 校验可表达字段，再删除重复手写解析；保留的窄 owner-specific 校验必须带移除条件。
5. 将 adapter-id metadata 接入 source validation 和 navigation input resolution，保持 inspect 输出聚焦配置来源事实。
6. 补充 tests/docs/schema/example 验证材料，覆盖 config inspect、config read、navigation resolution parity、adapter-id native option namespace、nested config shape 和 diagnostics。
7. 运行局部 Rust 测试、OpenSpec validation；跨 crate 或 docs/schema/example 同步完成后运行 workspace verifier。

Rollback 策略：由于 change 不改变 protocol/readable output 和 adapter handler payload，若迁移引入回归，可以临时保留旧内部 config loading helper，但不得恢复 `set|get|unset|list` 作为 public surface，也不得保留会绕过 typed-field metadata 的新 public behavior。

## Open Questions

已收敛：`docnav config` 长期只保留只读 `inspect`；`options.<adapter-id>.<option-key>` 使用现有 adapter registry id；旧裸 `options.<key>` 不兼容不迁移；inspect scope 聚焦 source inspection。无未回答开放问题，可以进入实现前审计。
