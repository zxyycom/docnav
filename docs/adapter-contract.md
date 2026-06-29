# 适配器契约

本文是格式适配器命令、标准参数消费边界、invoke、manifest 和 probe 的主规范。

## 命令

每个适配器提供：

```text
outline
read
find
info
invoke
manifest
probe
```

普通 CLI、readable JSON 和 `invoke` request 在各自输入解码后进入 canonical document operation input 或等价 semantic request，并复用业务逻辑；它们不复用输出包装或展示形态。`readable-view`（默认）和 `readable-json` 以阅读为主；`invoke` 和 `protocol-json` 属于完整协议接口，不以可读性为目标。
文档操作的直接 CLI 支持 `readable-view`（默认）、`readable-json` 和 `protocol-json` 输出；`manifest`、`probe` 和 `protocol-json` 输出各自专属机器 schema。
适配器可复用 SDK 的直接 CLI 基础能力完成通用命令分发、标准参数消费、adapter 配置读取、protocol request 构造、输出分流和错误通道投影。SDK direct CLI 的 argv 映射、配置字段映射、来源合并、校验和 metadata 由 [标准参数](standard-parameters.md) 定义；SDK 继续拥有命令分发、诊断投影与刷新、operation build 和最终 exit behavior。格式 adapter 只声明格式原生 CLI flag、native option registration、operation binding 和业务语义，并保留这些 options 的 ref 策略和 readable payload 字段语义。

适配器直接 CLI argv 必须复用 [标准参数](standard-parameters.md#输入与配置映射) 定义的 direct input 映射与兼容规则。

`manifest`、`probe` 和文档操作 `protocol-json` 的 stdout 仍使用本文件定义的专属机器 schema；存在可投影为 CLI warning 的诊断记录时按直接 CLI 规则写 stderr。`--help` 和子命令 help 只输出可纠错参数说明，不执行文档导航业务。共享 helper 的复用验收标准是保持这些 schema、plain text、stderr boundary 和 exit behavior 不变；document output owner 见 [输出模式](output.md#输出层边界)。

## 适配器职责

- 使用成熟 parser 解析格式。
- 生成扁平 outline 和可读 ref。
- 定义格式原生导航参数和直接 CLI 默认值。
- 返回有限结果和下一页 page。
- 按自身契约解析 ref 并读取，将非法 ref、无匹配 ref 等失败写入错误通道，并由边界层投影为对应 surface error。
- 在 invoke 中返回紧凑原始协议结果。

adapter 直接提供本格式的 ref、结构化 item facts、内容、结构化成本、info metadata 和 page，供 `docnav` 原样映射到原始协议，并由输出层派生阅读输出。

## Manifest

`manifest --output protocol-json` 返回稳定 manifest：

```text
manifest_version
adapter.id
adapter.name
adapter.version
formats[].id
formats[].extensions[]
formats[].content_types[]
capabilities[]
```

manifest 只接受 adapter 身份、支持格式、扩展名、content type 和 capabilities 字段，不声明协议范围或格式默认参数。manifest 字段扩展必须先由本文件和 manifest schema 定义。Markdown v0 adapter 必须声明并实现 `outline`、`read`、`find` 和 `info` 全部能力。

Markdown v0 adapter 的默认参数和 native option registration 属于 `docnav-markdown` 标准参数声明：默认 `pagination.enabled: true`、`limit: 6000`，格式原生 `options.max_heading_level: 3`。这些值不进入 manifest；direct CLI 和 `invoke` 分别按 [标准参数](standard-parameters.md) 定义的入口规则解析配置、默认值和 request `arguments`。
Markdown find 返回的 match ref 可按共享调用流程原样传给 read；没有局部导航区域时，可以返回 adapter 定义的全文 ref。find 的 ref 归属策略和 read 对该 ref 的接受与解释行为，由 [Markdown Adapter](adapters/markdown.md) 定义。`max_heading_level` 等格式原生 options 只影响 adapter 的导航粒度。

## Probe

probe 只识别格式，不执行导航。probe 输入只包含 path；`docnav` 在调用 probe 前解析 `--adapter` 或 core 推断得到的预选 adapter，但 adapter 选择提示不会作为 probe 参数传入。机器结果包含：

```text
probe_version
adapter_id
path
supported
format
confidence
reasons[]
```

每次判断至少包含一个 reason。不支持或内容不匹配时返回 `supported: false` 并给出 reason。`docnav` 必须以 probe 结果为准，不能只凭 adapter id、扩展名或 manifest 静默选中。

## Invoke

`invoke` 是独立的 protocol stdin/stdout 入口。SDK 将解码后的 stdin JSON value 作为 direct input 交给标准参数/typed-field processing；request envelope、operation、document path 和 raw `arguments` 都从该 direct input 中映射、校验并产出诊断交接数据。缺失的已注册参数可由 adapter invoke 入口的配置或默认值补足。未映射字段不由标准参数层解释；adapter 入口可通过标准参数返回的透传处理结果按自身策略保留、丢弃或交给 adapter-owned 语义校验。

`invoke` stdin JSON 使用 direct input validation，而不是 CLI argv 的 ignored-token warning 规则。Malformed JSON 属于 stdin transport decode failure；已解码 JSON value 直接进入标准参数/typed-field processing。缺少 envelope 必需字段、已出现已注册参数类型错误或其它 direct input validation failure 不得被 warning 后忽略，也不得暴露为 safe operation values。未映射 `arguments` 字段不由标准参数层解释；adapter 层作为最终消费者，可以通过透传处理结果丢弃这些字段，或在 adapter-owned native option 语义中返回协议错误。通过标准参数解析的 `outline/read/find/info` request 必须与 direct CLI 文档操作共享语义归一、request 构造或统一 operation handler，不得维护第二套业务参数解释规则。

SDK 可以复用 `docnav-protocol` 的 JSON decode、typed-field metadata 和标准参数 processing helper；failure surface 仍必须是由错误通道记录投影出的 protocol-shaped failure response。`invoke` stdin JSON 不是直接 CLI argv，按 [原始协议](protocol.md#schema-所有权) 的 direct input 边界验收。

适配器必须：

- 校验 `protocol_version` 字段和请求 schema。
- 只处理一个请求。
- stdout 只返回原始协议 envelope。
- 为分页操作返回下一页页码，结束时返回 null。
- 按自身声明的 `limit` 预算分页；ref 不得截断。outline/find 单条记录超过预算时，可以压缩 adapter-owned `label`、`summary`、`excerpt`、`cost` 或 `metadata` 等补充事实，但必须保留最小非空 `label` 并让分页前进。
- 不输出 CLI 阅读文本。

## 标准参数消费边界

- Adapter direct CLI 和 adapter `invoke` 的配置字段映射、来源标记、合并顺序、默认值和 schema metadata 由 [标准参数](standard-parameters.md) 定义。
- SDK 调用方必须提供 adapter id、registration、入口策略、内置默认值、native option specs 和可选默认用户配置目录；未提供默认用户配置目录时，SDK 按标准参数配置映射规则使用启动 cwd。
- SDK document operation help 必须展示配置路径覆盖参数，但 help 不读取 adapter direct CLI 配置。
- SDK document operation 必须按标准参数机制处理显式 argv、配置源和默认值；不可用配置源产生可投影为 `adapter_config_source_skipped` 的可恢复诊断，并继续按其余来源合并。
- manifest 只声明 adapter 能力，不提供默认参数。
- `docnav` 按自身标准参数 registration 和入口策略解析 core 通用参数。
- 格式原生 `options` 对 `docnav` 和接入层保持 opaque。
- Adapter native options 只有在对应 registration 声明时才参与标准参数解析；否则按 adapter-owned policy 消费、丢弃或报错。
- page 不属于配置默认值；入口省略 page 时固定从 `1` 开始。

## 协议字段对齐

`docnav` 不在 adapter 选择阶段做协议版本协商。候选适配器的 manifest 和 probe 输出必须通过 schema、必需字段、字段类型和语义校验；字段缺失、字段类型不符、shape 不对齐、语义校验失败、进程不可用或 `supported: false` 时，`docnav` 必须能形成包含 adapter id、阶段和原因的候选失败证据。候选遍历策略由 [架构](architecture.md#adapter-选择) 定义；选择成功或全部候选失败后的输出映射由 `docnav` 输出层负责。

选定 adapter 后的 `invoke` 响应不再属于候选选择阶段。`invoke` 响应必须通过 protocol response schema、必需字段、字段类型、operation/result shape 和语义校验；校验失败时返回 adapter/protocol 错误投影，不能把已经选定 adapter 的 invoke 失败当作普通候选失败继续静默切换。

原始协议字段对齐要求 adapter 使用 [原始协议](protocol.md#紧凑语义结果) 定义的结构化 item、`cost.measurements[]` 和 info facts。`display`、成本摘要和 info 摘要由 [输出模式](output.md) 的 readable projection 派生；adapter 不在 protocol result 中返回这些 readable-only 字段。

正式 schema：

- [manifest.schema.json](schemas/manifest.schema.json)
- [probe-result.schema.json](schemas/probe-result.schema.json)
