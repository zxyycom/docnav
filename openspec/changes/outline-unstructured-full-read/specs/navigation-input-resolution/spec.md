本 delta 定义 navigation input resolution 如何产出非结构化全文 outline 的标准调用参数，并在 adapter 正常 outline dispatch 前执行命中检查。

## ADDED Requirements

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
