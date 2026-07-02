# 适配器契约

本文是格式适配器源码级 library interface、静态 descriptor、标准参数消费边界、manifest metadata 和 probe 的主规范。默认文档操作通过 core release 内置 adapter handle 调用。

## 内置 adapter 接口

每个默认适配器作为 core release 内置 workspace crate 暴露 `docnav-adapter-contracts::Adapter` handle，并由 `docnav` static registry 注册。该注册项是源码级静态 descriptor：声明 adapter id、manifest metadata、capabilities、native option registry entries、operation binding 和 handler handle。当前最小 interface 使用 operation handler 粒度：

```text
manifest
probe
execute outline
execute read
execute find
execute info
```

`docnav-navigation` 把 core 已解析的 document operation input 构造成内部 protocol request，并通过选定 adapter handle 调用对应 operation。`readable-view`（默认）和 `readable-json` 以阅读为主；`protocol-json` 属于完整协议输出，不以可读性为目标。三种输出模式复用同一 adapter 业务结果，但不复用输出包装或展示形态。

格式 adapter 只在静态 descriptor 中声明格式原生 native option registry entries、native CLI/config/protocol source spelling、operation binding 和业务语义，并保留这些 options 的 ref 策略和 readable payload 字段语义。CLI argv 映射、配置字段映射、来源合并、标准参数校验、native option handoff 和 metadata 由 [标准参数](standard-parameters.md) 定义；adapter-owned option semantics 由 consuming adapter 校验，request construction、输出模式和最终 exit behavior 由 core/navigation/output owner 处理。

`manifest` 和 `probe` 是 adapter handle 上的 metadata/support methods，不是独立默认 CLI 命令。`docnav adapter list` 可以展示 manifest metadata；adapter selection 必须以 static registry membership、capability 和 probe 结果为准。Document output owner 见 [输出模式](output.md#输出层边界)。

## 适配器职责

- 使用成熟 parser 解析格式。
- 生成扁平 outline 和可读 ref。
- 定义格式原生导航参数、源码级 native option registry entries、adapter-side option validation 和内置默认值。
- 返回有限结果和下一页 page。
- 按自身契约解析 ref 并读取，将非法 ref、无匹配 ref 等失败写入错误通道，并由边界层投影为对应 surface error。
- 返回紧凑原始协议语义结果。

adapter 直接提供本格式的 ref、结构化 item facts、内容、结构化成本、info metadata 和 page，供 `docnav` 原样映射到原始协议，并由输出层派生阅读输出。

## manifest 元数据

Adapter manifest metadata 包含稳定字段：

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

Markdown v0 adapter 的默认参数和 native option registry entries 属于 core-linked `docnav-markdown` 静态 descriptor 和标准参数声明：默认 `pagination.enabled: true`、`limit: 6000`，格式原生 `options.max_heading_level: 3`。这些值不进入 manifest；core document commands 按 [标准参数](standard-parameters.md) 定义的入口规则解析配置、默认值和 request arguments。
Markdown find 返回的 match ref 可按共享调用流程原样传给 read；没有局部导航区域时，可以返回 adapter 定义的全文 ref。find 的 ref 归属策略和 read 对该 ref 的接受与解释行为，由 [Markdown Adapter](adapters/markdown.md) 定义。`max_heading_level` 等格式原生 options 只影响 adapter 的导航粒度。

## probe 识别

probe 只识别格式，不执行导航。probe 输入只包含 path；`docnav` 在调用 probe 前解析 `--adapter` 或 core 推断得到的预选 adapter，但 adapter 选择提示不会作为 probe 参数传入。probe result 包含：

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

## 文档操作执行边界

`docnav-navigation` 的职责限于构造内部 operation request，并 dispatch selected adapter operation handler。Adapter implementation source 由 core release 的 static registry 和 linked workspace crate 决定。

Adapter handle 接收的输入已经通过 core input/config boundary、adapter selection 和 selected-adapter native option projection；它只消费 typed operation input 和 merged native option values。Core、protocol 和 CLI owner 处理 stdin/stdout、CLI argv、exit code、unknown envelope fields、malformed JSON 和 public input token classification。Native option 的 type/range failure 属于 adapter consumption diagnostic。

Adapter operation handler 必须：

- 只处理当前 request 指定的一个 operation。
- 为分页操作返回下一页页码，结束时返回 null。
- 按自身声明的 `limit` 预算分页；ref 不得截断。outline/find 单条记录超过预算时，可以压缩 adapter-owned `label`、`summary`、`excerpt`、`cost` 或 `metadata` 等补充事实，但必须保留最小非空 `label` 并让分页前进。
- 返回结构化 operation result 或 adapter diagnostic；CLI 阅读文本、stdout/stderr 和最终 exit code 由 core/output owner 处理。

## 标准参数消费边界

本节只说明 adapter descriptor 与标准参数之间的交接；来源合并和字段映射的完整规则由 [标准参数](standard-parameters.md) 拥有。

- Core static registry 和 adapter descriptor 提供 adapter id、入口策略、内置默认值、native option registry entries 和 native option public source spelling。
- 标准参数 pipeline 使用这些源码级信息准备 standard typed operation arguments、source info、诊断交接数据和 merged native option handoff。
- `docnav-navigation` 只消费已解析出的 operation input；配置源读取、public input 分类、输出通道和 exit code 属于 core/CLI/output owner。
- manifest 只声明 adapter 身份、格式和能力，不提供默认参数。
- 格式原生 `options` 对 `docnav` 和调用入口保持 opaque；core 只在 adapter selection 后按 selected adapter descriptor 做支持性投影。
- 不属于 selected adapter 的 option 返回 unsupported native option diagnostic；type mismatch、range invalid 和格式语义由选中 adapter 在消费时诊断。
- 显式 adapter id 不存在时，adapter selection diagnostic 优先于任何 option validation。
- page 不属于配置默认值；入口省略 page 时固定从 `1` 开始。

## 协议字段对齐

`docnav` 不在 adapter 选择阶段做协议版本协商。候选适配器的 manifest metadata 和 probe result 必须通过必需字段、字段类型和语义校验；字段缺失、字段类型不符、shape 不对齐、语义校验失败、adapter layer 不可用或 `supported: false` 时，`docnav` 必须能形成包含 adapter id、阶段和原因的候选失败证据。候选遍历策略由 [架构](architecture.md#adapter-选择) 定义；选择成功或全部候选失败后的输出映射由 `docnav` 输出层负责。

选定 adapter 后的 operation result 不再属于候选选择阶段。Operation result 必须通过 protocol response schema、必需字段、字段类型、operation/result shape 和语义校验；校验失败时返回 adapter/protocol 错误投影，不能把已经选定 adapter 的执行失败当作普通候选失败继续静默切换。

原始协议字段对齐要求 adapter 使用 [原始协议](protocol.md#紧凑语义结果) 定义的结构化 item、`cost.measurements[]` 和 info facts。`display`、成本摘要和 info 摘要由 [输出模式](output.md) 的 readable projection 派生；adapter 不在 protocol result 中返回这些 readable-only 字段。

正式 schema：

- [manifest.schema.json](schemas/manifest.schema.json)
- [probe-result.schema.json](schemas/probe-result.schema.json)
