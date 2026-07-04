# Navigation Input Resolution

本文是 document navigation input resolution 的主规范。读者应能从本文判断 `docnav` core 和 `docnav-navigation` 的 owner 边界、navigation command 的输入来源如何进入解析流程、raw project/user config source 如何加载、通用字段声明与 selected adapter declarations 如何在使用点注册并合并参与解析，以及最终如何构造 `RequestEnvelope` / `OperationArguments` 并调用 adapter。

配置只是 navigation 参数来源之一。本文不把配置文件、CLI argv、protocol arguments 或 adapter defaults 单独提升为主 owner；这些来源都在 `docnav-navigation` 的 input resolution 流程中按统一规则解析。

## Owner 边界

### `docnav` core

Core 负责 invocation 入口和非 navigation 命令：

1. 解析命令类型，区分 navigation command、`config`、`init`、`doctor`、`version`、help 和 adapter inspection。
2. 为 navigation command 提供 project config 和 user config 的 source descriptor/path，包括默认路径、显式 override 路径和 project root/user config 定位结果。
3. 对非 navigation 命令在 core 内完成处理，不进入 navigation input resolution。
4. 对 navigation 命令，把 raw command、config source descriptors/paths 和当前 core release 的 adapter registry 交给 `docnav-navigation`。
5. 接收 navigation outcome，并按 [输出模式](output.md)、[原始协议](protocol.md) 和 [CLI](cli.md) 规则进行 surface 投影与退出码映射。

Core 不为 navigation command 预先读取 raw config JSON、完成参数来源合并、native option enrichment、selected-adapter projection 或 request construction。

### `docnav-navigation`

`docnav-navigation` 是 navigation input resolution、raw navigation config source loading、adapter selection、request construction 和 adapter dispatch 的 owner：

1. 根据 core 提供的 config source descriptors/paths 加载 raw project/user config sources，保留路径、缺失状态、读取失败和原始 JSON value。
2. 从 raw navigation command 和 raw config sources 中解析 routing 必需输入，例如 operation、document path、declared adapter intent、ref/query 和 direct parameter sources。
3. 使用 adapter registry 和 routing 输入选择 selected adapter；adapter selection 规则见 [适配器契约](adapter-contract.md#adapter-选择)。
4. 构造 operation field set：注册通用 operation 参数，并把 selected adapter 的 `AdapterOptionSpec` 注册进同一个 typed-field set，保留 adapter-owned native options、内置默认值和 source binding metadata。
5. 将 explicit command value、project config、user config 和 built-in default 作为来源集合，按 `explicit > project > user > built_in` 解析每个声明参数。
6. 使用 typed-field 的底层校验和提取函数得到 typed 参数；解析失败时返回带来源信息的 blocking diagnostic。
7. 构造 `RequestEnvelope` / `OperationArguments`，并通过 selected adapter handle 调用对应 operation。

Adapter handler 接收的是已解析 typed arguments。参数基础校验和提取发生在 input resolution 阶段，不由 handler 再消费 raw source value。

## 输入来源

Navigation input resolution 接收四类来源：

| 来源 | 提供方 | 用途 |
| --- | --- | --- |
| explicit | raw navigation command | 本次调用显式传入的 operation、path、ref/query、page、limit、output、adapter intent 和 native option source |
| project | `docnav-navigation` 从 project config descriptor/path 加载的 source | 项目级 document operation 默认值和 adapter-owned native option source |
| user | `docnav-navigation` 从 user config descriptor/path 加载的 source | 用户级 document operation 默认值和 adapter-owned native option source |
| built_in | `docnav-navigation` 注册后的 selected adapter declarations 和 core navigation defaults | 缺省 pagination、limit、output、page 和 adapter 参数默认值 |

Project config 路径为 `<project-root>/.docnav/docnav.json`。User config descriptor/path 由 core 用户配置位置提供。`docnav-navigation` 加载这些 descriptor/path 指向的 raw source：默认路径缺失表示该来源 absent；显式存在但不可读、JSON 无效或顶层不是 object 是 config source failure。

`path`、`ref` 和 `query` 是当前调用的 direct navigation input，不从配置文件取得。`page` 是 continuation call-position state，不是配置字段；入口省略时使用 built-in `1`。

## 配置文件形状

Config source 只表达 document operation 默认值和 adapter-owned native option source，不拥有最终参数解析流程。

已文档化字段：

| 字段 | 含义 |
| --- | --- |
| `defaults.adapter` | 默认 declared adapter id。 |
| `defaults.pagination.enabled` | 默认分页状态。 |
| `defaults.pagination.limit` | 默认分页预算，必须是正整数。 |
| `defaults.output` | 默认输出模式。 |
| `options.*` | Adapter-owned native option source value。 |

`defaults.limit` 和 `defaults.page` 不是合法字段。未知顶层字段、未知 `defaults.*` 字段、未知 `defaults.pagination.*` 字段和无法由当前 registry 声明接收的 `options.*` source 都是 input resolution failure。

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
  declare common fields + register selected adapter native option declarations
  collect explicit/project/user/built_in source candidates
  resolve declared parameters with explicit > project > user > built_in
  validate/extract typed values through typed-field helpers
  bind typed values into OperationArguments
  construct RequestEnvelope
  dispatch selected adapter operation handler
```

Resolution 必须保持来源信息，便于诊断表达 explicit、project config、user config 或 built-in default。Config source 缺失不产生诊断；present invalid source、unknown field、unmapped public input、type/range invalid 和 missing required value 产生 blocking diagnostic。

当 `pagination.enabled` 最终为 `false` 时，resolution 在构造 operation arguments 前把分页预算归一为最大正整数预算。该归一化不回写 raw command、config source 或 protocol JSON。

## Selected Adapter 参数声明

通用 operation 字段由 `docnav-navigation` 声明；selected adapter 的 typed-field declarations 是 adapter-owned native option 参数事实源。Adapter 只声明这些字段；`docnav-navigation` 在构造 operation field set 时按 selected adapter 和 operation 主动注册。Adapter declaration 至少提供：

- 参数 identity、owner、`options.<key>` final arguments path 和 operation applicability。
- explicit input、project config、user config 和 built-in default 的 processing/source 映射。
- value kind、required/optional/nullability、allowed values、range、static default 和 dynamic default metadata。
- operation binding metadata，用于把 typed values 写入 `OperationArguments`。

`options.*` 保持 adapter-owned namespace。多个 adapter 或多个 type variant 可以声明同名 option key；resolution 不把同名 key 折叠为 core-owned 字段。Adapter selection 完成后，只解析 selected adapter 声明接收的 native option sources。

Typed-field helper 负责字段级校验和 typed extraction。Adapter 可以通过声明表达格式语义约束，但 handler 不再接收未校验 raw option value 来执行基础类型或范围校验。`docnav-navigation` 只合并通用字段和 selected adapter 注册的字段，并执行来源解析、校验、提取和 `OperationArguments` 绑定；它不从 config key 或独立 registry entry 重新推导 adapter-owned value kind、range 或 default 语义。Config unknown-option detection 使用当前 operation field set 的 typed-field processing metadata 判断哪些 `options.*` source 已被声明接收。

## Request Construction

Resolution 完成后，`docnav-navigation` 构造内部 protocol request：

| operation | `OperationArguments` |
| --- | --- |
| `outline` | `limit`、`page`、`options` |
| `read` | `ref`、`limit`、`page`、`options` |
| `find` | `query`、`limit`、`page`、`options` |
| `info` | `options` |

最终 `limit` 和 `page` 必须是正整数。`ref` 和 `query` 是 operation-owned required input；缺失或非法时在 navigation boundary 返回 input diagnostic。`options` 是 selected adapter 的 typed native option object；原始协议只承载该对象，不解释格式语义。

## 错误出口

| 位置 | 结果 |
| --- | --- |
| Core command classification | 非 navigation 命令不进入本文流程；help/version 不读取 document config。 |
| Config source loading | `docnav-navigation` 将默认路径缺失视为 absent；存在但不可读、JSON 无效或顶层非 object 返回 config source diagnostic。 |
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
5. `path`、`ref`、`query` 和 `page` 不进入配置文件字段集合。
6. Typed-field 校验/提取先于 `RequestEnvelope` / `OperationArguments` 构造。
7. 解析结果不回写 raw command、config source、schema 示例或 protocol JSON fixture。
