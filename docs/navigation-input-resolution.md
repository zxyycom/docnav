# Navigation Input Resolution

本文是 document navigation input resolution 的主规范。读者应能从本文判断 `docnav` core 和 `docnav-navigation` 的 owner 边界、navigation command 的输入来源如何进入解析流程、raw project/user config source 如何加载、通用字段声明与 selected adapter definition declarations 如何在使用点注册并合并参与解析，以及最终如何构造 `RequestEnvelope` / `OperationArguments` / `NativeOptionHandoff` 并通过 selected adapter definition dispatch。

配置只是 navigation 参数来源之一。本文不把配置文件、CLI argv、protocol arguments 或 adapter defaults 单独提升为主 owner；这些来源都在 `docnav-navigation` 的 input resolution 流程中按统一规则解析。

## Owner 边界

### `docnav` core

Core 负责 invocation 入口和非 navigation 命令：

1. 解析命令类型，区分 navigation command、`config`、`init`、`doctor`、`version`、help 和 adapter inspection。
2. 为 navigation command 提供 project config 和 user config 的 source descriptor，包括 source level、resolved path 和 path origin。
3. 对非 navigation 命令在 core 内完成处理，不进入 navigation input resolution。
4. 对 navigation 命令，把 raw command、config source descriptors/paths 和当前 core release 的 adapter registry 交给 `docnav-navigation`。
5. 接收 navigation outcome，并按 [输出模式](output.md)、[原始协议](protocol.md) 和 [CLI](cli.md) 规则进行 surface 投影与退出码映射。

Core 不为 navigation command 预先读取 raw config JSON、完成参数来源合并、native option enrichment、selected-adapter projection 或 request construction。
例外是 core-owned runtime invocation logging surface：core 可以读取所选 project/user config 文件中的 `invocation_log` section 来初始化独立日志 sink，但该 section 不属于 navigation parameter source，不参与 selected adapter declaration 注册，也不得写入 `RequestEnvelope` / `OperationArguments`。

### `docnav-navigation`

`docnav-navigation` 是 navigation input resolution、raw navigation config source loading、adapter selection、request construction 和 adapter dispatch 的 owner：

1. 根据 core 提供的 config source descriptors/paths 加载 raw project/user config sources，保留路径、缺失状态、读取失败和原始 JSON value。
2. 从 raw navigation command 和 raw config sources 中解析 routing 必需输入，例如 operation、document path、declared adapter intent、ref/query 和 direct parameter sources。
3. 使用 adapter registry 和 routing 输入选择 selected adapter；adapter selection 规则见 [适配器契约](adapter-contract.md#adapter-选择)。
4. 构造 operation field set：注册通用 operation 参数，并从 selected adapter definition 注册当前 operation 适用的 `AdapterOptionSpec`，保留 adapter-owned native options、内置默认值、source binding metadata 和 handler binding metadata。
5. 将 explicit command value、project config、user config 和 built-in default 作为来源集合，按 `explicit > project > user > built_in` 解析每个声明参数。
6. 使用 typed-field 的底层校验和提取函数得到 typed 参数；解析失败时返回带来源信息的 blocking diagnostic。
7. 构造 `RequestEnvelope` / `OperationArguments` 和 handler-facing selected adapter `NativeOptionHandoff`，并通过 selected adapter definition 调用对应 operation handler。

Adapter handler 接收的是已解析 typed operation arguments 和 selected adapter native option handoff。参数基础校验和提取发生在 input resolution 阶段，不由 handler 再消费 raw source value、raw CLI argv 或 raw config JSON。

## 输入来源

Navigation input resolution 接收四类来源：

| 来源 | 提供方 | 用途 |
| --- | --- | --- |
| explicit | raw navigation command | 本次调用显式传入的 operation、path、ref/query、page、limit、output、adapter intent 和 native option source |
| project | `docnav-navigation` 从 project config descriptor/path 加载的 source | 项目级 document operation 默认值和 adapter-owned native option source |
| user | `docnav-navigation` 从 user config descriptor/path 加载的 source | 用户级 document operation 默认值和 adapter-owned native option source |
| built_in | `docnav-navigation` 注册后的 selected adapter definition declarations 和 core navigation defaults | 缺省 pagination、limit、output、page 和 adapter 参数默认值 |

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

Config source 只表达 document operation 默认值和 adapter-owned native option source，不拥有最终参数解析流程。

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
| `options.*` | Adapter-owned native option source value。 |

`outline.mode_rules[].mode` 只能是 `structured` 或 `unstructured_full`。`outline.mode_rules[].path` 使用 Rust `regex` matcher pattern，匹配使用 `/` 分隔的规范化 `document.path`，且 pattern 必须匹配整个路径。项目根内文档使用 project-relative path，项目根外文档使用规范化绝对 path。当前文档化 pattern syntax 是 Rust regex crate 支持的正则表达式，例如 `docs/raw\.md`、`notes/raw/.+\.md` 和 `.*\.txt`；它不是 glob、gitignore 规则或 shell expansion。Unsupported matcher feature、无法编译的 pattern、缺少 `path` / `mode` 或未知 mode 都是 source-scoped input resolution failure。

`invocation_log.*` 字段只声明配置文件 shape；日志 enablement、path 规范化、content capture root、写入失败降级和输出通道由 [CLI](cli.md#invocation-logging) 与 [输出模式](output.md) 拥有。`defaults.limit` 和 `defaults.page` 不是合法字段。未知顶层字段、未知 `defaults.*` 字段、未知 `defaults.pagination.*` 字段、未知 `outline.*` 字段、未知 `invocation_log.*` 字段和无法由当前 registry 声明接收的 `options.*` source 都是 input resolution failure。

配置 schema 只用于示例校验和编辑器提示；runtime 不要求先加载 schema。

## Resolution 流程

Navigation command 的主流程：

```text
docnav core
  parse command type
  if non-navigation: handle in core
  if navigation: pass raw command + config source descriptors/paths + registry

docnav-navigation
  load raw project/user config sources
  parse routing-required input
  select adapter from registry
  declare common fields + register selected adapter definition native option declarations
  collect explicit/project/user/built_in source candidates
  resolve declared parameters with explicit > project > user > built_in
  validate/extract typed values through typed-field helpers
  bind typed values into OperationArguments + NativeOptionHandoff
  construct RequestEnvelope
  dispatch selected adapter definition operation handler
```

Resolution 必须保持来源信息，便于诊断表达 explicit、project config、user config 或 built-in default。Default-path config source 缺失不产生 diagnostic；explicit config path missing、present invalid source、unknown field、unmapped public input、type/range invalid 和 missing required value 产生 blocking diagnostic。

Invocation logging 可观察 resolution 阶段的稳定 metadata，但不改变 resolution outcome。可记录的 navigation-owned metadata 包括 adapter selection success/failure layer、selected adapter id、request construction success/failure、request id 或 fallback correlation id availability、operation arguments 的 bounded shape、selected adapter dispatch start/end status 和 output/error status metadata。不得把日志 event 作为额外 resolution result 返回给 caller。

Config path flag 只影响 source descriptor 的 resolved path 和 path origin，不成为 navigation parameter source value。显式选择的 config file 内部字段仍以 `project` 或 `user` source level 参与 `explicit > project > user > built_in` 合并；direct argv value 始终优先于 project config，project config 始终优先于 user config。

Config source diagnostic details 必须携带 source level 和 selected config file path，使 project config 与 user config 在路径都由 CLI flag 选择时仍可区分。

当 `pagination.enabled` 最终为 `false` 时，resolution 在构造 operation arguments 前把分页预算归一为最大正整数预算。该归一化不回写 raw command、config source 或 protocol JSON。

## Outline Mode Resolution

`outline` operation 在标准调用参数中包含 navigation-owned `outline_mode`。合法值为 `structured` 和 `unstructured_full`，默认值为 `structured`。`outline_mode` 不是 adapter 私有 option、ref policy、raw protocol argument 或 public CLI override flag。

`outline_mode` resolution 发生在 document path 规范化、adapter selection、通用字段声明和 selected adapter declaration registration 之后，且早于 selected adapter 的正常 outline handler dispatch。优先级固定为：

1. `outline.mode_rules[]` path selector。
2. `outline.auto_full_read.thresholds[]` adapter-scoped cost threshold selector。
3. built-in default `structured`。

Path selector 只在 `outline` operation 中生效。User config rules 保持文件顺序；project config rules 保持文件顺序并在 user rules 之后评估；最后一个匹配当前规范化 document path 的 rule 胜出。匹配结果可以显式产出 `structured`，此时 cost threshold selector 不得覆盖该 path rule。

Cost threshold selector 只在没有 path rule 产出 `outline_mode` 时运行。Navigation 必须先按 selected adapter id 过滤 `outline.auto_full_read.thresholds[]`；没有 selected-adapter candidate threshold 时保持 `structured`，且不得调用 selected adapter definition 的 full-read capability group。存在 candidate threshold 时，navigation 按 `unit` 合并阈值，同一 `unit` 取最小正整数 `value`，并只把这些 effective units 传给 selected adapter definition 声明的 full-read cost measurement hook/declaration。

Threshold 比较只使用 selected adapter full-read capability group 返回的标准 `Cost.measurements[]`。Adapter definition 未声明、无法安全返回 measurement，或返回结果中没有 effective threshold 的 `unit` 时，selector 不命中并保持 `structured`。Navigation 不解析格式私有内容，也不为 adapter 缺失的 unit 发明成本语义。

当 `outline_mode` 为 `unstructured_full` 时，navigation 在正常 outline handler 前进入非结构化全文读取路径，通过 selected adapter definition 的可选 full-read capability group 或默认 UTF-8 原文读取方案产出非结构化 outline success result。该路径不返回 entries、ref、page 或 continuation；cost threshold 只是 selector，不是 `limit`、`page` 或 content 截断预算。

## Selected Adapter 参数声明

通用 operation 字段由 `docnav-navigation` 声明；selected adapter definition 的 typed-field declarations 是 adapter-owned native option 参数事实源。Adapter 只在 definition/factory 中声明这些字段；`docnav-navigation` 在构造 operation field set 时按 selected adapter definition 和 operation 主动注册。Adapter declaration 至少提供：

- 参数 identity、owner、`options.<key>` final arguments path 和 operation applicability。
- explicit input、project config、user config 和 built-in default 的 processing/source 映射。
- value kind、required/optional/nullability、allowed values、range、static default 和 dynamic default metadata。
- operation binding metadata，用于把 typed values 写入 `OperationArguments` 并构造 handler-facing `NativeOptionHandoff`。

`options.*` 保持 adapter-owned namespace。多个 adapter 或多个 type variant 可以声明同名 option key；resolution 不把同名 key 折叠为 core-owned 字段。Adapter selection 完成后，只解析 selected adapter definition 声明接收的 native option sources。未被 selected definition 声明接收的 owner-scoped `options.*` source 必须严格失败，不进入 dispatch。

Typed-field helper 负责字段级校验和 typed extraction。Adapter 可以通过声明表达格式语义约束，但 handler 不再接收未校验 raw option value 来执行基础类型或范围校验。`docnav-navigation` 只合并通用字段和 selected adapter definition 注册的字段，并执行来源解析、校验、提取、`OperationArguments` 绑定和 `NativeOptionHandoff` 构造；它不从 config key、CLI flag 或独立 registry entry 重新推导 adapter-owned value kind、range 或 default 语义。Config unknown-option detection 使用当前 operation field set 的 typed-field processing metadata 判断哪些 `options.*` source 已被声明接收。

## Request Construction

Resolution 完成后，`docnav-navigation` 构造内部 protocol request：

| operation | `OperationArguments` |
| --- | --- |
| `outline` | `limit`、`page`、`options` |
| `read` | `ref`、`limit`、`page`、`options` |
| `find` | `query`、`limit`、`page`、`options` |
| `info` | `options` |

最终 `limit` 和 `page` 必须是正整数。`ref` 和 `query` 是 operation-owned required input；缺失或非法时在 navigation boundary 返回 input diagnostic。`options` 是 selected adapter 的 typed native option object；原始协议只承载该对象，不解释格式语义。Handler-facing `NativeOptionHandoff` 从同一 typed resolution result 派生，保留 identity、owner、namespace、key、source、type metadata 和 typed JSON value；它是内部 dispatch contract，不改变 `protocol-json`、`readable-json` 或 `readable-view` 输出 shape。

Invocation log 中的 request construction metadata 只能引用 `RequestEnvelope` 的 correlation facts 和 bounded argument summaries。`ref` 和 `query` 摘要不得替代 adapter-owned ref/query 语义；日志实现不能为了记录日志而重新解析 ref、复制 adapter native option 语义或改变 request construction failure classification。

## 错误出口

| 位置 | 结果 |
| --- | --- |
| Core command classification | 非 navigation 命令不进入本文流程；help/version 不读取 document config。 |
| Config source loading | `docnav-navigation` 将 default path missing 视为 absent；explicit path missing、unreadable、JSON 无效或顶层非 object 返回 blocking config source diagnostic；present default-path source unreadable、JSON 无效或顶层非 object 也返回 config source diagnostic。 |
| Source mapping | 未知字段、旧字段名、operation 不适用参数、unmapped input 或 selected adapter 不接收的 native option source 返回 input diagnostic。 |
| Typed-field validation/extraction | 类型、范围、allowed value、required/nullability 和 default invalid 返回带来源的 typed validation diagnostic。 |
| Adapter selection | Declared adapter lookup/probe 失败或 automatic discovery 全部失败按 [适配器契约](adapter-contract.md#adapter-选择) 返回 selection diagnostic。 |
| Request construction | 绑定 metadata 缺失、arguments shape invalid 或 envelope construction failure 返回 internal/navigation diagnostic。 |

本文只定义 input resolution 的失败位置和 owner；protocol failure shape 见 [协议错误对象](protocol.md#协议错误对象)，readable failure shape 见 [输出模式](output.md)，退出码见 [CLI](cli.md#退出码)。

## 维护注意事项

维护本文时保持这些不变量：

1. Core 只分流命令并提供 config source descriptors/paths；navigation command 的 raw config source loading、来源合并、typed validation、request construction 和 adapter dispatch 由 `docnav-navigation` 拥有。
2. Config 是参数来源之一，不是 navigation input resolution 的 owner。
3. 只有 selected adapter 注册的 typed-field option 字段参与 native option resolution。
4. 来源优先级固定为 `explicit > project > user > built_in`。
5. Config source descriptor 必须保留 source level、resolved path 和 path origin；config path flag selection 不改变参数来源优先级。
6. `path`、`ref`、`query` 和 `page` 不进入配置文件字段集合。
7. Typed-field 校验/提取先于 `RequestEnvelope` / `OperationArguments` 构造。
8. 解析结果不回写 raw command、config source、schema 示例或 protocol JSON fixture。
