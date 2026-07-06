# navigation-input-resolution Specification

## Purpose
定义 document navigation input resolution 的长期契约：`docnav` core 只解析命令类型，并为 navigation command 提供 config source descriptors/paths 与 registry；raw project/user config source loading、routing 输入解析、adapter selection、selected adapter typed-field 参数声明读取、来源优先级、typed validation/extraction、operation argument binding、request construction 和 adapter dispatch 由 `docnav-navigation` 拥有。
## Requirements
### Requirement: Core only hands raw navigation inputs to the navigation layer
`docnav` core MUST classify each invocation before navigation input resolution. For non-navigation commands, core MUST keep the owner-defined command behavior in core. For navigation commands, core MUST pass the raw command, project/user config source descriptors/paths and current static adapter registry to `docnav-navigation`.

#### Scenario: Navigation command enters input resolution
- **WHEN** caller executes `docnav outline docs/guide.md`
- **THEN** core classifies the invocation as a navigation command
- **THEN** core supplies project/user config source descriptors/paths
- **THEN** core passes raw command, config source descriptors/paths and registry to `docnav-navigation`
- **THEN** `docnav-navigation` loads raw project/user config sources

#### Scenario: Non-navigation command stays in core
- **WHEN** caller executes help, version, init, doctor, config without document context, or static adapter inspection
- **THEN** core handles that command in its owner boundary
- **THEN** the command does not enter navigation input resolution

### Requirement: Navigation input resolution selects the adapter before parameter extraction
`docnav-navigation` MUST parse routing-required input, select the adapter from the current static registry, then read only the selected adapter's typed-field parameter declarations for source resolution.

#### Scenario: Declared adapter selects one registry entry
- **WHEN** direct input or config source declares an adapter id
- **THEN** `docnav-navigation` looks up that id in the current static registry
- **THEN** selection succeeds only if the selected adapter probe accepts the document
- **THEN** failure returns adapter selection diagnostics without falling back to later registry entries

#### Scenario: Automatic discovery uses registry order
- **WHEN** no declared adapter id exists
- **THEN** `docnav-navigation` traverses static registry entries in release order
- **THEN** the first adapter probe returning `supported: true` selects that adapter
- **THEN** later adapters are not part of the selected parameter declaration set

### Requirement: Selected adapter typed-field declarations own parameter facts
Selected adapter typed-field declarations MUST provide parameter identity, owner, namespace/key, operation applicability, source mappings, default metadata, value kind, requiredness/nullability, constraints and operation argument binding metadata. `docnav-navigation` MUST use those declarations as the parameter fact source.

#### Scenario: Typed-field metadata drives validation
- **WHEN** selected Markdown declares `options.max_heading_level` with integer range `1..6`
- **AND** a config or direct source provides `options.max_heading_level`
- **THEN** navigation input resolution validates and extracts the typed value before request construction
- **THEN** the Markdown handler receives the typed value rather than the raw source value

#### Scenario: Same option key can have different owners
- **WHEN** multiple selected-adapter declarations expose native option sources with the same public key across owners or type variants
- **THEN** navigation input resolution preserves owner, namespace, key and type variant metadata
- **THEN** it does not collapse those declarations into one core-owned parameter

### Requirement: Source priority is explicit over project over user over built-in
Navigation input resolution MUST combine available source candidates with fixed priority `explicit > project > user > built_in`. Missing default config sources MUST be absent. Present invalid config sources MUST produce blocking diagnostics and MUST NOT be recovered by lower-priority sources.

#### Scenario: Direct input wins
- **WHEN** the same parameter has explicit input, project config, user config and built-in default candidates
- **THEN** navigation input resolution uses the explicit value
- **THEN** the resolved source info records explicit input

#### Scenario: Project config wins over user config
- **WHEN** a parameter has project config and user config candidates but no explicit value
- **THEN** navigation input resolution uses the project config value
- **THEN** the resolved source info records project config

#### Scenario: Built-in default fills absence
- **WHEN** a declared parameter has no explicit, project or user source
- **AND** the selected adapter declaration or navigation default provides a built-in default
- **THEN** navigation input resolution validates and uses that default

### Requirement: Config sources are source inputs, not the owner
Project and user config files MUST be modeled as raw source inputs loaded by `docnav-navigation` from core-supplied descriptors/paths. Config shape, known config paths and unmapped-field behavior MUST be interpreted through selected declarations and source mapping rules; config files MUST NOT own final parameter resolution, request construction or adapter option validation.

#### Scenario: Config JSON maps to declared paths
- **WHEN** project config contains a value at a selected declaration's config path
- **THEN** navigation input resolution maps that value to the declared parameter identity
- **THEN** validation and extraction use the same typed-field declaration as explicit input

#### Scenario: Unknown config field is blocking input
- **WHEN** present config contains an unknown or unmapped public field
- **THEN** navigation input resolution returns a source-scoped blocking diagnostic
- **THEN** it does not silently pass the field to an adapter handler

### Requirement: Request construction consumes typed resolution results
`docnav-navigation` MUST bind resolved typed values into `OperationArguments`, construct a `RequestEnvelope`, and dispatch the selected adapter operation handler. Request construction MUST NOT mutate raw command input, raw config source objects or protocol examples.

#### Scenario: Read arguments are bound from typed values
- **WHEN** selected input resolution has typed `ref`, `limit`, `page` and `options` values for `read`
- **THEN** request construction writes those values to read `OperationArguments`
- **THEN** source info remains available for diagnostics and audit

#### Scenario: Disabled pagination finalizes before dispatch
- **WHEN** resolved `pagination.enabled` is false
- **THEN** navigation input resolution finalizes the effective limit before request construction
- **THEN** the final value is not written back to raw argv or config source JSON

### Requirement: Diagnostics preserve owner and source
Navigation input resolution MUST report missing required values, unmapped public input, config source failures, unsupported selected-adapter options, type mismatch, range invalid and operation-inapplicable parameters as blocking diagnostics with source attribution.

#### Scenario: Invalid native option is rejected before handler dispatch
- **WHEN** selected adapter declarations reject a native option due to unsupported operation, wrong type or range invalid
- **THEN** navigation input resolution returns a diagnostic with selected adapter/source metadata
- **THEN** the adapter handler is not called with that invalid raw value

#### Scenario: Adapter selection precedes option validation
- **WHEN** declared adapter id is missing from the static registry
- **AND** the same request contains an invalid-looking native option
- **THEN** navigation input resolution returns adapter selection diagnostics
- **THEN** option validation for that adapter does not run

### Requirement: Adapter native options 必须是 explicit owner-scoped input sources
Adapter native options MUST 表达为 explicit owner-scoped input sources. `docnav-navigation` MUST know which selected-adapter source locations can contain adapter-owned options, and MUST validate/extract those values through selected adapter typed-field declarations before handler execution.

Unknown direct input、unknown config fields 和 undeclared native options 默认 MUST 产生 blocking diagnostics。只有 selected adapter typed-field declarations 声明 option namespace 并拥有校验规则时，native option value MAY enter request construction.

#### Scenario: 已声明 native option 进入 selected adapter typed-field 校验
- **WHEN** core CLI、config 或 protocol request input 包含已声明的 adapter native option
- **THEN** navigation input resolution records it as an adapter-owned native option source
- **THEN** selected adapter typed-field validation/extraction validates or rejects that option before handler execution

#### Scenario: 未声明 native option 返回输入诊断
- **WHEN** core CLI、config 或 protocol request input 包含 undeclared native option
- **THEN** navigation input resolution returns input diagnostic
- **THEN** request 在 handler execution 前返回

### Requirement: Navigation resolution 产出标准 outline_mode
`docnav-navigation` MUST 在 outline operation 的标准调用参数中表达 `outline_mode`。该字段 MUST 至少支持 `structured` 和 `unstructured_full` 两个值，默认值 MUST 为 `structured`。`outline_mode` MUST 由 config source 中的 ordered path rules、adapter-scoped cost threshold 或 built-in default 产出，并 MUST 在 adapter 正常 outline dispatch 前成为标准调用参数。

Resolution priority MUST be deterministic: path rules > adapter-scoped cost thresholds > built-in default. This change MUST NOT expose a public CLI outline-mode override flag or raw protocol argument for normal callers.

#### Scenario: 默认 outline mode 保持结构化 dispatch
- **WHEN** 调用方执行 outline
- **AND** 没有 path rule 匹配当前 document path
- **AND** 没有 adapter-scoped cost threshold 触发
- **THEN** navigation 得到 `outline_mode: "structured"`
- **THEN** navigation 继续调用 selected adapter 的正常 outline handler

#### Scenario: path rule 匹配解析为非结构化全文 outline
- **WHEN** 调用方执行 outline
- **AND** `outline.mode_rules[]` 中最后一个匹配当前 document path 的 rule 指定 `mode: "unstructured_full"`
- **THEN** navigation 得到 `outline_mode: "unstructured_full"`
- **THEN** navigation 在 selected adapter 正常 outline handler 之前进入非结构化全文读取路径
- **THEN** result reason 为 `path_rule`

### Requirement: outline.mode_rules 使用确定性 path pattern 批量选择
Config source MAY 提供 `outline.mode_rules[]` 作为 path batch selector。每条 rule MUST 包含 `path` 和 `mode`。`path` MUST 是由维护中的 matcher implementation 支持、并由主规范/schema 明确记录的 path pattern，匹配 `/` 分隔的规范化 `document.path`；项目根内文档使用 project-relative path，项目根外文档使用规范化绝对 path。`mode` MUST 是 `structured` 或 `unstructured_full`。Rule evaluation MUST be deterministic: user rules keep file order, project rules keep file order and are evaluated after user rules, and the last matching rule wins.

The path pattern syntax itself is not the product goal of this change. Implementation MUST use a standard-library capability or maintained matcher library to compile and match patterns, and MUST NOT implement a custom glob/regex/gitignore parser. Unsupported matcher features, invalid pattern syntax and unknown mode values MUST return source-scoped input resolution diagnostics.

#### Scenario: Project rule 覆盖 user rule
- **WHEN** user config 中较早 rule 匹配 `docs/raw-note.md` 并指定 `mode: "structured"`
- **AND** project config 中较晚 rule 匹配同一路径并指定 `mode: "unstructured_full"`
- **THEN** navigation 产出 `outline_mode: "unstructured_full"`

#### Scenario: 后写 rule 覆盖先写 rule
- **WHEN** 同一 config source 中两个 rules 都匹配 `docs/raw-note.md`
- **AND** 后一个 rule 指定 `mode: "structured"`
- **THEN** navigation 产出 `outline_mode: "structured"`

#### Scenario: 无效 path rule 返回输入诊断
- **WHEN** config source 中的 `outline.mode_rules[]` rule 缺少 `path` 或 `mode`
- **OR** rule 指定 unknown `mode`
- **OR** rule 的 `path` 不能被选定的 matcher implementation 编译
- **OR** rule 使用未被 documented matcher syntax 支持的 feature
- **THEN** navigation 返回 source-scoped input resolution diagnostic
- **THEN** diagnostic 指向对应 rule 的字段位置
- **THEN** navigation 不退回到自定义 matcher、静默忽略该 rule 或调用 selected adapter 的正常 outline handler

### Requirement: adapter-scoped cost threshold 可以触发非结构化全文 outline
Config source MAY 提供 `outline.auto_full_read.thresholds[]` 作为 cost threshold selector。每条 threshold MUST 包含 `adapter`、`unit` 和正整数 `value`，表达“当 selected adapter id 等于 `adapter`，且 selected adapter 声明的 full-read cost measurement 中同名 `unit` 小于等于 `value` 时直接全文读取”。该 selector MUST only run when no path rule produced `outline_mode`.

Threshold evaluation MUST avoid unconditional cost work. Navigation MUST first filter `outline.auto_full_read.thresholds[]` to entries whose `adapter` equals the selected adapter id. If no threshold matches the selected adapter, navigation MUST keep `outline_mode: "structured"` and MUST NOT call the selected adapter full-read cost measurement hook.

For selected-adapter candidate thresholds, navigation MUST merge thresholds by `unit`. When multiple candidate thresholds use the same `unit`, the effective threshold value for that unit MUST be the minimum positive `value` among them. Different units MUST remain independently comparable. Navigation MUST pass only the effective requested units to the selected adapter full-read cost measurement hook/declaration. Threshold evaluation MUST use standard `Cost.measurements[]` produced by that hook/declaration. When selected adapter does not declare or cannot produce measurements, the measurement set is empty. Navigation MUST compare only the effective `unit/value` pairs; it MUST NOT parse format-private content or invent adapter-specific cost semantics.

Cost threshold is a selector, not a limiter. When it triggers `outline_mode: "unstructured_full"`, the successful outline result MUST contain the complete content. `limit`, `page` and the threshold MUST NOT truncate the returned content.

#### Scenario: adapter cost threshold 命中后直接全文读取
- **WHEN** config source provides `outline.auto_full_read.thresholds[]`
- **AND** no path rule produced `outline_mode`
- **AND** selected adapter id matches at least one threshold `adapter`
- **AND** navigation merges selected-adapter candidate thresholds by `unit`, using the minimum `value` for each unit
- **AND** selected adapter declares a full-read cost measurement for at least one effective threshold `unit`
- **AND** navigation obtains that measurement safely
- **AND** at least one measurement value is less than or equal to the effective threshold `value` for its unit
- **THEN** navigation 得到 `outline_mode: "unstructured_full"`
- **THEN** navigation returns complete content without entries、ref、page or continuation
- **THEN** result reason 为 `cost_threshold`

#### Scenario: 超过 adapter cost threshold 保持结构化 outline
- **WHEN** config source provides `outline.auto_full_read.thresholds[]`
- **AND** no path rule produced `outline_mode`
- **AND** selected adapter id matches at least one threshold `adapter`
- **AND** navigation merges selected-adapter candidate thresholds by `unit`, using the minimum `value` for each unit
- **AND** selected adapter declares and returns measurements for effective threshold units
- **AND** every returned measurement value is greater than the effective threshold `value` for its unit
- **THEN** navigation 得到 `outline_mode: "structured"`
- **THEN** navigation 继续调用 selected adapter 的正常 outline handler

#### Scenario: path structured rule 可以关闭 cost 自动全文
- **WHEN** config source provides `outline.auto_full_read.thresholds[]`
- **AND** a matching path rule specifies `mode: "structured"`
- **THEN** navigation 得到 `outline_mode: "structured"`
- **THEN** navigation does not use cost thresholds to override that path rule
- **THEN** navigation 继续调用 selected adapter 的正常 outline handler

#### Scenario: threshold adapter 不匹配时保持结构化 outline
- **WHEN** config source provides `outline.auto_full_read.thresholds[]`
- **AND** no path rule produced `outline_mode`
- **AND** no threshold entry matches selected adapter id
- **THEN** navigation 得到 `outline_mode: "structured"`
- **THEN** navigation 不调用 selected adapter 的 full-read cost measurement hook
- **THEN** navigation 继续调用 selected adapter 的正常 outline handler

#### Scenario: threshold unit 未被 adapter measurement 命中时保持结构化 outline
- **WHEN** config source provides a threshold matching selected adapter id
- **AND** navigation merges selected-adapter candidate thresholds by `unit`
- **AND** selected adapter full-read cost measurements do not contain any effective threshold `unit`
- **THEN** navigation 得到 `outline_mode: "structured"`
- **THEN** navigation 继续调用 selected adapter 的正常 outline handler

#### Scenario: 同单位 threshold 合并使用最小值
- **WHEN** user config provides threshold `{ adapter: "docnav-markdown", unit: "tokens", value: 3000 }`
- **AND** project config provides threshold `{ adapter: "docnav-markdown", unit: "tokens", value: 1200 }`
- **AND** selected adapter id is `docnav-markdown`
- **AND** no path rule produced `outline_mode`
- **THEN** effective threshold for unit `tokens` is `1200`
- **THEN** navigation requests at most one `tokens` full-read measurement from the selected adapter cost measurement hook

#### Scenario: 无法安全取得 threshold measurement 时保持结构化 outline
- **WHEN** only cost threshold could trigger non-structured full-read
- **AND** navigation cannot safely obtain full-read cost measurements at runtime
- **THEN** navigation 得到 `outline_mode: "structured"`
- **THEN** navigation 继续调用 selected adapter 的正常 outline handler
- **THEN** navigation does not return lossy or binary content

### Requirement: Navigation pre-dispatch 检查必须早于正常 adapter outline
当 `outline_mode` 为 `unstructured_full` 时，`docnav-navigation` MUST NOT 调用 selected adapter 的正常 outline handler。该路径 MUST 产出非结构化 outline success result，或在 path-triggered default 读取/adapter hook 无法安全产出文本时返回受控诊断。

#### Scenario: 命中后跳过正常 outline handler
- **WHEN** selected adapter 已完成 selection、probe 和参数解析
- **AND** `outline_mode` 为 `unstructured_full`
- **THEN** navigation 不调用 selected adapter 的正常 outline handler
- **THEN** navigation 通过 adapter 可选 full-read hook 或默认 UTF-8 原文读取方案产出非结构化 outline result

### Requirement: Navigation loads config sources from descriptor paths with origin-aware absence semantics

`docnav-navigation` MUST treat project and user config files as raw source inputs loaded from core-supplied descriptors. Each descriptor MUST preserve the source level, resolved path and whether the path came from explicit CLI input or a default path. Missing default-path config sources MUST be absent without diagnostics. Missing, unreadable, invalid JSON, or non-object config sources selected through explicit CLI config path flags MUST produce a blocking config source diagnostic. Config path flag selection MUST NOT become a navigation parameter source value and MUST NOT alter the parameter merge priority `explicit > project > user > built_in`.

#### Scenario: Explicit config path failure is blocking

- **WHEN** core supplies a project or user config source descriptor selected by `--project-config` or `--user-config`
- **AND** the descriptor path is missing, unreadable, invalid JSON, or a top-level non-object JSON value
- **THEN** `docnav-navigation` returns a blocking config source diagnostic for that source level and path
- **THEN** lower-priority config sources or built-in defaults do not mask that source failure

#### Scenario: Default config path absence is non-blocking

- **WHEN** core supplies a project or user config source descriptor selected by default path resolution
- **AND** the descriptor path does not exist
- **THEN** `docnav-navigation` treats that config source as absent
- **THEN** it continues resolving declared parameters from remaining sources and built-in defaults

#### Scenario: Config path selection is separate from parameter priority

- **WHEN** core supplies explicit config file paths and the selected project config, selected user config and direct argv all provide candidates for the same declared parameter
- **THEN** `docnav-navigation` resolves the parameter value from direct argv first
- **THEN** project config still overrides user config
- **THEN** user config still overrides built-in defaults
- **THEN** the fact that a config file path was selected by CLI flag does not turn values inside that file into direct argv values

#### Scenario: Diagnostics preserve selected config path

- **WHEN** a present project or user config source contains an unknown field, unsupported selected-adapter option, type mismatch or range-invalid value
- **THEN** `docnav-navigation` reports the source level and selected config file path in the diagnostic details
- **THEN** diagnostics distinguish project config from user config even when both paths were supplied through CLI flags

