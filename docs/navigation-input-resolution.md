# Navigation Input Resolution

本文是 document navigation input resolution 的主规范。读者应能从本文判断 `docnav` core parameter catalog 与 `docnav-navigation` 的 owner 边界、navigation command 的输入来源如何进入解析流程、raw project/user config source 如何加载和完整校验、selected adapter/current-operation view 如何从同一 catalog 过滤，以及一个 typed resolution result 如何投影为 protocol arguments、closed strategy input 和 core output facts。

配置只是 navigation 参数来源之一。本文不把配置文件、CLI argv、protocol arguments 或 adapter defaults 单独提升为主 owner；这些来源都在 `docnav-navigation` 的 input resolution 流程中按统一规则解析。

## Owner 边界

### `docnav` core

Core 负责 invocation 入口和非 navigation 命令：

1. 解析命令类型，区分 navigation command、`config`、`init`、`doctor`、`version`、help 和 adapter inspection。
2. 为 navigation command 提供 project config 和 user config 的 source descriptor，包括 source level、resolved path 和 path origin。
3. 对非 navigation 命令在 core 内完成处理，不进入 navigation input resolution。
4. 对 navigation 命令，把 operation、固定 positional/path facts、normalized document CLI `Source`、config source descriptors/paths、core-owned `DocumentParameterCatalog` 和当前 core release 的 adapter registry 交给 `docnav-navigation`。
5. 接收 navigation outcome，并按 [输出模式](output.md)、[原始协议](protocol.md) 和 [CLI](cli.md) 规则进行 surface 投影与退出码映射。

Core 不为 navigation command 预先读取 raw config JSON、完成参数来源合并、selected-operation projection 或 request construction。Core 是 caller-configurable document operation parameter catalog 的唯一 author；adapter definition 不增加 catalog entry。
例外是 core-owned runtime invocation logging surface：core 可以读取所选 project/user config 文件中的 `invocation_log` section 来初始化独立日志 sink，但该 section 不属于 navigation parameter catalog，也不得写入 `RequestEnvelope` / `OperationArguments`。

### `docnav-navigation`

`docnav-navigation` 是 navigation input resolution、raw navigation config source loading、adapter selection、request construction 和 adapter dispatch 的 owner：

1. 根据 core 提供的 config source descriptors/paths 加载 raw project/user config sources，保留路径、缺失状态、读取失败和原始 JSON value。
2. 从 fixed command facts、normalized CLI candidates 和 raw config sources 中解析 routing 必需输入，例如 operation、document path、declared adapter intent 和 ref/query。
3. 使用 adapter registry 和 routing 输入选择 selected adapter；adapter selection 规则见 [适配器契约](adapter-contract.md#adapter-选择)。
4. 使用 full catalog 校验完整 config shape、known adapter namespace、已声明 path 和 typed source value；再从 catalog 过滤当前 operation 的 common entry 与 exact selected-adapter-tag entry，构造 selected `FieldDefSet`。
5. Fixed positional input 由 navigation-private direct mapping 产出；core 提供的 normalized CLI `Source` 已保留 canonical field identity、locator、typed/invalid input 和 source attribution。Project/user JSON 通过 Serde companion 按同一 catalog metadata 提取 declared candidates。
6. 将 selected `FieldDefSet` 与 explicit、project、user sources交给 resolution core；字段有 environment locator 时还可加入 declared-only env source，优先级为 `explicit > env > project > user > built_in`，否则保持 `explicit > project > user > built_in`。Static default 由 field metadata 自动回退。
7. Resolution core 执行 merge、provenance 和最终 canonical field validation；selected/contributing invalid candidate 或 missing required value 返回带来源信息的 blocking diagnostic，materialization 不返回部分参数对象。
8. 从同一个 `ResolutionResult` 构造 protocol `Options` / `OperationArguments`、closed `StandardOperationInput` 和 core output projection，并通过 selected adapter definition 调用对应 operation strategy。

Adapter strategy 接收的是已解析的 operation-specific closed typed input。Source resolution、merge/default、标准类型 materialization 和 core-configured validation 已完成；strategy 不消费 raw source、generic parameter bag 或 protocol envelope，但可以为算法正确性防御性地校验或重复校验格式语义。

### Document CLI candidate 边界状态

**Current：** core 在 argv parsing 前从 `DocumentParameterCatalog` 构造 operation-scoped CLI `FieldDefSet`；core 交回 fixed positional/path facts、config descriptors/paths、catalog、registry 和 normalized typed/invalid CLI `Source`。Navigation 完成 routing/selection 后，从同一 catalog 构造 current-operation common + exact selected-adapter-tag `FieldDefSet`。只有 selected-set member 进入 priority、merge、validation 与 materialization；任何不属于该 set 的 explicit candidate 都在 request construction/dispatch 前严格失败。Full catalog 仍用于完整 config validation 和其它 known adapter namespace 判断。

CLI set、full validation view 和 selected-operation set 是同一 catalog 的不同投影，不是多套参数模型。完整 runtime 调用关系见 [架构](architecture.md#document-named-option-派生状态)。

## 输入来源

Navigation input resolution 接收以下来源：

| 来源 | 提供方 | 用途 |
| --- | --- | --- |
| explicit | fixed command facts + normalized CLI `Source` | 本次调用显式传入的 operation、path、ref/query，以及保留 canonical identity 的 page、limit、output、adapter intent 和 adapter-scoped parameter candidates |
| env（逐字段可选） | declared-only env extractor | 仅对存在 environment locator 的 catalog field 产生 candidate；当前 catalog inventory 没有启用 env locator |
| project | `docnav-navigation` 从 project config descriptor/path 加载的 source | 项目级 document operation 默认值和 adapter-scoped parameter source |
| user | `docnav-navigation` 从 user config descriptor/path 加载的 source | 用户级 document operation 默认值和 adapter-scoped parameter source |
| built_in | core catalog static defaults | 缺省 pagination、limit、output、page 和 adapter-scoped 参数默认值 |

Environment locator presence 是 activation gate：有 locator 的字段才可提取 env candidate，并使用 `explicit > env > project > user > built_in`；没有 locator 时，即使存在同名环境变量也不产生 candidate，优先级保持 `explicit > project > user > built_in`。为 catalog field 新增或删除 env locator 是可观察的产品输入变更。

Core-supplied project/user config descriptor 至少包含：

- source level：`project` 或 `user`。
- resolved path：core 为本次 invocation 选择的 exact config JSON file path。
- path origin：路径来自 explicit CLI flag 还是 default path resolution。Project default 来自当前 project context 的 `.docnav/docnav.json`；user defaults 来自 `DOCNAV_CONFIG_DIR/docnav.json` 或平台用户默认 `.docnav/docnav.json`。所有非 explicit origin 都按 default path origin 处理缺失语义。

`docnav-navigation` 加载这些 descriptor/path 指向的 raw source：default path missing 表示该来源 absent，不产生 diagnostic。Explicit path selected by `--project-config` or `--user-config` missing、unreadable、invalid JSON 或 top-level non-object 是 blocking config source diagnostic。Present default-path source 如果 unreadable、invalid JSON 或 top-level non-object，也作为 config source diagnostic 处理。

`path`、`ref` 和 `query` 是当前调用的 direct navigation input，不从配置文件取得。`page` 是 continuation call-position state，不是配置字段；入口省略时使用 built-in `1`。

Runtime invocation logging 可以记录这些来源的 bounded metadata，例如 selected config source descriptor、adapter selection outcome、request id availability、operation、page/limit、path display 和 selected adapter dispatch status。
Document path metadata 只记录 bounded display 和 hash。
Ref/query metadata 可以记录 presence/length/hash/preview。日志不得记录完整 raw config source、完整 direct query/ref、完整 protocol request/response 或 adapter-owned ref grammar 作为跨层稳定身份。

## 配置文件形状

Config source 只表达 document operation 默认值和 adapter-scoped parameter source，不拥有最终参数解析流程。

已文档化字段：

| 字段 | 含义 |
| --- | --- |
| `defaults.adapter` | 默认 declared adapter id。 |
| `defaults.pagination.enabled` | 默认分页状态。 |
| `defaults.pagination.limit` | 默认分页预算，必须是正整数。 |
| `defaults.output` | 默认输出模式。 |
| `outline.mode_rules[]` | `outline` 的 path selector。每条 rule 包含 `path` 和 `mode`。 |
| `outline.auto_full_read.thresholds[]` | `outline` 的 adapter-scoped cost threshold selector。每条 threshold 包含 `adapter`、`unit` 和正整数 `value`。 |
| `invocation_log.enabled` | Core-owned runtime invocation logging 显式启用开关。 |
| `invocation_log.path` | Core-owned JSONL invocation log file path。 |
| `invocation_log.content_capture.enabled` | Core-owned content capture 显式启用开关。 |
| `invocation_log.content_capture.root` | Core-owned content capture root directory path。 |
| `options.<adapter-id>.<option-key>` | Core-catalog adapter-scoped parameter source value。`<adapter-id>` 是 exact static adapter tag，`<option-key>` 是 catalog entry 的 config key。 |

`outline.mode_rules[].mode` 只能是 `structured` 或 `unstructured_full`。`outline.mode_rules[].path` 使用 Rust `regex` matcher pattern，匹配使用 `/` 分隔的规范化 `document.path`，且 pattern 必须匹配整个路径。项目根内文档使用 project-relative path，项目根外文档使用规范化绝对 path。当前文档化 pattern syntax 是 Rust regex crate 支持的正则表达式，例如 `docs/raw\.md`、`notes/raw/.+\.md` 和 `.*\.txt`；它不是 glob、gitignore 规则或 shell expansion。Unsupported matcher feature、无法编译的 pattern、缺少 `path` / `mode` 或未知 mode 都是 source-scoped input resolution failure。

`invocation_log.*` 字段只声明配置文件 shape；日志 enablement、path 规范化、content capture root、写入失败降级和输出通道由 [CLI](cli.md#invocation-logging) 与 [输出模式](output.md) 拥有。`defaults.limit` 和 `defaults.page` 不是合法字段。未知顶层字段、未知 `defaults.*` 字段、未知 `defaults.pagination.*` 字段、未知 `outline.*` 字段、未知 `invocation_log.*` 字段、未知 adapter id namespace、selected adapter namespace 下未被 selected operation 声明接收的 `options.<adapter-id>.*` source，以及旧裸 `options.<option-key>` source 都是 input resolution failure。旧裸 `options.<option-key>` 不做兼容读取、迁移或 adapter id 推断，只按普通 unknown/invalid config path 处理。

配置 schema 只用于示例校验和编辑器提示；runtime 不要求先加载 schema。

## Core Parameter Catalog

Core-owned `DocumentParameterCatalog` 是 caller-configurable document operation 参数的唯一 declaration source。Current inventory 为：

| 参数 | Scope / consumer |
| --- | --- |
| `page` | common；paged strategy input |
| `limit` | common；paged strategy input |
| `pagination.enabled` | common；navigation-owned effective-limit binding |
| `output` | common；core output projection |
| Markdown `max_heading_level` | exact `docnav-markdown` tag；outline/find strategy input |

每个 entry 组合 canonical field identity、已启用的 CLI/env/config locator、standard value kind、constraints/default、merge strategy、operation binding、closed consumer binding 和可选 exact static adapter-id tag。Untagged entry 对适用 operation 是 common；tagged entry 只在 tag 等于 selected adapter id 时进入 selected-operation view。Catalog construction 在 runtime parsing 前拒绝 duplicate/incompatible identity、locator、unknown adapter id、missing/incompatible consumer binding 和 invalid operation binding。

`docnav-typed-fields` 提供 canonical `FieldDef` / `FieldDefSet` mechanics；`cli-config-resolution` 执行 ordered source resolution、merge、fallback、provenance 和 all-or-nothing materialization；Serde companion 和 env extractor只按 catalog locator 提取 source candidates。它们不拥有产品 inventory 或 consumer binding。Adapter definition 不参与 parameter declaration/discovery。

Full config validation view 接受 full catalog 的所有 known config locators，用于区分 known-other adapter namespace、unknown adapter/path 和 source typed failure。Selected-operation view 只包含 current operation 的 common entry 与 exact matching tag entry，用于 applicability、resolution 和 binding。`options.<adapter-id>.<option-key>` 是 core catalog 的 adapter-scoped config locator；同名 key 不跨 adapter namespace 合并。Fixed `path`、`ref`、`query` 使用 navigation-private direct processing，不进入 catalog。

## Resolution 流程

Navigation command 的 **Current** 主流程：

```text
docnav core
  parse command type
  if non-navigation: handle in core
  if navigation: build static/generated command from operation-scoped catalog view
  extract normalized typed/invalid CLI Source
  pass fixed command facts + CLI Source + config source descriptors/paths + catalog + registry

docnav-navigation
  load raw project/user config sources
  validate source shape/keys through full-catalog view and owner-specific shape validation
  parse routing-required input
  select adapter from registry
  build selected FieldDefSet from current-operation common + exact matching adapter tag
  reject explicit candidates outside the selected/current-operation set
  map fixed positional input and extract declared env/project/user candidates
  resolve enabled sources in priority order, then built_in static defaults
  merge and perform final canonical field validation/materialization
  derive protocol Options/OperationArguments + closed StandardOperationInput + core output
  construct RequestEnvelope and PreparedNavigationRequest
  dispatch selected adapter definition operation strategy
```

Resolution 必须保持来源信息，便于诊断表达 explicit、project config、user config 或 built-in default。Default-path config source 缺失不产生 diagnostic；explicit config path missing、present invalid source、unknown field、unmapped public input、type/range invalid 和 missing required value 产生 blocking diagnostic。

Config source validation 与 selected adapter/operation resolution 是两阶段边界：

1. Full validation 阶段读取 project/user config source，使用 full catalog 和 owner-specific shape validation 报告 unknown field、unknown adapter id、nested shape failure 和 declared scalar typed value failure。Known-other adapter namespace 可以通过 full validation。
2. Selected adapter/operation 阶段只消费 current operation 的 common fields 与 exact selected-adapter-tag fields。其它已知 adapter namespace 不进入 selected resolution 或 strategy input；selected namespace 下不适用于 current operation 的 option 是 blocking unsupported parameter diagnostic。

Invocation logging 可观察 resolution 阶段的稳定 metadata，但不改变 resolution outcome。可记录的 navigation-owned metadata 包括 adapter selection success/failure layer、selected adapter id、request construction success/failure、request id 或 fallback correlation id availability、operation arguments 的 bounded shape、selected adapter dispatch start/end status 和 output/error status metadata。不得把日志 event 作为额外 resolution result 返回给 caller。

Config path flag 只影响 source descriptor 的 resolved path 和 path origin，不成为 navigation parameter source value。显式选择的 config file 内部字段仍以 `project` 或 `user` source level 参与 `explicit > project > user > built_in` 合并；direct argv value 始终优先于 project config，project config 始终优先于 user config。

Config source diagnostic details 必须携带 source level 和 selected config file path，使 project config 与 user config 在路径都由 CLI flag 选择时仍可区分。

当 `pagination.enabled` 最终为 `false` 时，resolution 在 sibling projections 前把分页预算归一为最大正整数 effective limit。该 effective limit 同时进入 protocol arguments 与 paged closed strategy input；`pagination.enabled` 本身不进入 adapter input。归一化不回写 fixed command facts、normalized CLI source 或 config source。

## Outline Mode Resolution

`outline` operation 在标准调用参数中包含 navigation-owned `outline_mode`。合法值为 `structured` 和 `unstructured_full`，默认值为 `structured`。`outline_mode` 不是 adapter 私有 option、ref policy、raw protocol argument 或 public CLI override flag。

`outline_mode` resolution 发生在 document path 规范化、adapter selection 和 selected-operation catalog resolution 之后，且早于 selected adapter 的正常 outline strategy dispatch。优先级固定为：

1. `outline.mode_rules[]` path selector。
2. `outline.auto_full_read.thresholds[]` adapter-scoped cost threshold selector。
3. built-in default `structured`。

Path selector 只在 `outline` operation 中生效。User config rules 保持文件顺序；project config rules 保持文件顺序并在 user rules 之后评估；最后一个匹配当前规范化 document path 的 rule 胜出。匹配结果可以显式产出 `structured`，此时 cost threshold selector 不得覆盖该 path rule。

Cost threshold selector 只在没有 path rule 产出 `outline_mode` 时运行。Navigation 必须先按 selected adapter id 过滤 `outline.auto_full_read.thresholds[]`；没有 selected-adapter candidate threshold 时保持 `structured`，且不得调用 selected adapter definition 的 full-read capabilities/hooks。存在 candidate threshold 时，navigation 按 `unit` 合并阈值，同一 `unit` 取最小正整数 `value`，并只把这些 effective units 传给 selected adapter definition 暴露的 full-read cost measurement hook。

Threshold 比较只使用 selected adapter full-read hook 返回的标准 `Cost.measurements[]`。Adapter definition 未声明、无法安全返回 measurement，或返回结果中没有 effective threshold 的 `unit` 时，selector 不命中并保持 `structured`。Navigation 不解析格式私有内容，也不为 adapter 缺失的 unit 发明成本语义。

当 `outline_mode` 为 `unstructured_full` 时，navigation 在正常 outline strategy 前进入非结构化全文读取路径，通过 selected adapter definition 的可选 full-read hooks 或默认 UTF-8 原文读取方案产出非结构化 outline success result。该路径不返回 entries、ref、page 或 continuation；cost threshold 只是 selector，不是 `limit`、`page` 或 content 截断预算。

## Selected Operation Catalog View

`docnav-navigation` 以 operation 过滤 catalog entries，再保留 untagged common entries 与 tag 精确等于 selected adapter id 的 entries。Non-matching tagged entry 不进入 selected `FieldDefSet`，也不向 strategy 暴露。当前 Markdown `max_heading_level` 只有在 selected adapter 为 `docnav-markdown` 且 operation 为 outline/find 时适用。

Selected `FieldDefSet` 负责来源解析、字段级校验和 typed materialization。Navigation 按 selected identity 过滤 normalized CLI source，映射 fixed positional input，提取 applicable env/project/user candidates并调用 resolver；它不从 config key、CLI flag、adapter definition 或 registry entry 重新推导 locator、value kind、range、merge、default 或 binding。Adapter strategy 不参与这一步，可以在收到 closed typed input 后补充格式算法语义校验。

## Request Construction

Resolution 完成后，`docnav-navigation` 从同一个 `ResolutionResult` 构造 sibling projections：

| operation | `OperationArguments` |
| --- | --- |
| `outline` | `limit`、`page`、`options` |
| `read` | `ref`、`limit`、`page`、`options` |
| `find` | `query`、`limit`、`page`、`options` |
| `info` | `options` |

Protocol projection 保持上表的 `OperationArguments` 和 stable serialized `Options` shape。Strategy projection 是 closed `StandardOperationInput::{Outline, Read, Find, Info}`，只包含 operation-specific strategy-visible typed facts；没有通用 lookup、source metadata、declaration 或 serialized protocol representation。Core projection保留 resolved output mode 和 navigation orchestration facts，`output` 不进入 strategy input。

最终 effective `limit` 和 `page` 必须是正整数。`ref` 和 `query` 是 operation-owned required input；缺失或非法时在 navigation boundary 返回 input diagnostic。Markdown `max_heading_level` 同时投影到 stable protocol `options` value 与 outline/find closed input；原始协议只承载该值，不解释格式算法语义。

Invocation log 中的 request construction metadata 只能引用 `RequestEnvelope` 的 correlation facts 和 bounded argument summaries。`ref` 和 `query` 摘要不得替代 adapter-owned ref/query 语义；日志实现不能为了记录日志而重新解析 ref、复制 adapter-scoped parameter 语义或改变 request construction failure classification。

## 错误出口

| 位置 | 结果 |
| --- | --- |
| Core command classification | 非 navigation 命令不进入本文流程；help/version 不读取 document config。 |
| Config source loading | `docnav-navigation` 将 default path missing 视为 absent；explicit path missing、unreadable、JSON 无效或顶层非 object 返回 blocking config source diagnostic；present default-path source unreadable、JSON 无效或顶层非 object 也返回 config source diagnostic。 |
| Source mapping | 未知字段、旧字段名、operation 不适用参数、unmapped input 或 selected-operation view 不接收的 adapter-scoped source 返回 input diagnostic。 |
| Typed-field validation/extraction | 类型、范围、allowed value、required/nullability 和 default invalid 返回带来源的 typed validation diagnostic。 |
| Adapter selection | Declared adapter lookup/probe 失败或 automatic discovery 全部失败按 [适配器契约](adapter-contract.md#adapter-选择) 返回 selection diagnostic。 |
| Request construction | 绑定 metadata 缺失、arguments shape invalid 或 envelope construction failure 返回 internal/navigation diagnostic。 |

本文只定义 input resolution 的失败位置和 owner；protocol failure shape 见 [协议错误对象](protocol.md#协议错误对象)，readable failure shape 见 [输出模式](output.md)，退出码见 [CLI](cli.md#退出码)。

## 维护注意事项

维护本文时保持这些不变量：

1. Core 只分流命令并提供 config source descriptors/paths；navigation command 的 raw config source loading、来源合并、typed validation、request construction 和 adapter dispatch 由 `docnav-navigation` 拥有。
2. Config 是参数来源之一，不是 navigation input resolution 的 owner。
3. 只有 current-operation common entries 与 exact selected-adapter-tag entries 参与 selected resolution。
4. 有 environment locator 的字段按 `explicit > env > project > user > built_in`；没有 locator 的字段不产生 env candidate，并保持 `explicit > project > user > built_in`。
5. Config source descriptor 必须保留 source level、resolved path 和 path origin；config path flag selection 不改变参数来源优先级。
6. `path`、`ref`、`query` 和 `page` 不进入配置文件字段集合。
7. Typed-field 校验/提取先于 protocol、closed strategy input 和 core output projections。
8. 解析结果不回写 fixed command facts、normalized CLI source、config source、schema 示例或 protocol JSON fixture。
