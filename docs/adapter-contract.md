# 适配器契约

本文定义格式适配器与 `docnav` core / `docnav-navigation` 的交接契约。它拥有源码级 library interface、静态 descriptor、默认 adapter layer invariant、manifest/probe、adapter 选择规则、operation dispatch、adapter-owned ref/result 边界和 selected-adapter typed argument handoff。

## 内置 adapter 接口

每个默认适配器作为 core release 内置 workspace crate 暴露 `docnav-adapter-contracts::Adapter` handle，并由 `docnav` static registry 注册。注册项是源码级静态 descriptor，声明 adapter id、manifest metadata、operation metadata、typed-field 参数声明、native option registry entries、operation binding 和 handler handle。

当前最小 interface 使用 operation handler 粒度：

```text
manifest
probe
execute outline
execute read
execute find
execute info
```

`manifest` 和 `probe` 是 adapter handle 上的 metadata/support methods。默认 adapter layer 的必需文档操作集合为 `outline`、`read`、`find` 和 `info`；进入默认 adapter layer 的 adapter 必须全部实现这些 handler。缺少任一 handler 属于 adapter layer invalid 或 release validation 问题，单次 adapter selection 只处理 registry lookup 和 probe outcome。

`docnav-navigation` 接收 core 交出的 raw navigation command、config source descriptors/paths 和 adapter registry，加载 raw project/user config sources，完成 navigation input resolution，构造内部 operation request，并通过 selected adapter handle dispatch 对应 operation。Adapter 返回结构化业务结果或 adapter diagnostic。

格式 adapter 在静态 descriptor 中声明格式原生 native option registry entries、typed-field 参数声明、operation binding、内置默认值 metadata 和 adapter-owned option semantics。本文定义 selected adapter 最终消费 typed operation arguments 的边界。Input resolution 规则见 [Navigation Input Resolution](navigation-input-resolution.md)。

## 适配器职责

- 使用成熟 parser 解析格式。
- 生成扁平 outline 和 adapter-owned ref。
- 定义格式原生导航参数、源码级 native option registry entries、typed-field 参数声明和内置默认值 metadata。
- 返回有限结果和下一页 page。
- 按自身契约解析 ref 并读取，将非法 ref、无匹配 ref 等失败返回为 adapter diagnostic。
- 返回符合 [原始协议](protocol.md#紧凑语义结果) 的紧凑语义结果。

adapter 直接提供本格式的 ref、结构化 item facts、内容、结构化成本、info metadata 和 page。

## manifest 元数据

Adapter manifest metadata 的字段范围是 adapter 身份和支持格式。稳定字段为：

```text
manifest_version
adapter.id
adapter.name
adapter.version
formats[].id
formats[].extensions[]
formats[].content_types[]
```

manifest 字段范围限定为 adapter 身份、支持格式、扩展名和 content type。manifest 字段扩展必须先由本文件和 manifest schema 定义。正式 schema 见 [manifest.schema.json](schemas/manifest.schema.json)。

格式默认值和 native option registry entries 属于 core-linked adapter descriptor 与 typed-field 参数声明。manifest 保持为 adapter 身份和格式 metadata。

## probe 识别

probe 的职责是格式识别。probe 输入只包含 path；adapter 选择提示保留在 selection 流程中。probe result 包含：

```text
probe_version
adapter_id
path
supported
format
confidence
reasons[]
```

每次判断至少包含一个 reason。内容匹配失败时返回 `supported: false` 并给出 reason。扩展名、content type 和其它格式识别线索属于 adapter probe 可使用的内部判断材料；`docnav` 的 traversal order 仍由 declared adapter lookup 或 automatic discovery registry order 决定。正式 schema 见 [probe-result.schema.json](schemas/probe-result.schema.json)。

## adapter 选择

Adapter selection 的输入是 resolved declared adapter id，或 declared adapter id 缺失状态。

Declared adapter id 表达 caller intent。存在 declared adapter id 时，`docnav` 使用 declared selection path。

存在 declared adapter id 时，`docnav-navigation` 在当前 core release 的 static registry 中查找同名 adapter，并执行该 adapter 的 probe。registry lookup 失败、probe result 契约无效或 probe 返回 `supported: false` 时，返回 adapter selection diagnostic；probe 返回 `supported: true` 时选中该 adapter。声明式选择的通过条件是同名 registry entry 和成功 probe。

不存在 declared adapter id 时，`docnav-navigation` 进入 automatic discovery。Automatic discovery 只按 static registry 顺序遍历 adapter 并执行 probe，返回第一个 `supported: true` 的 adapter。

遍历过程中，单个 adapter 的 probe 契约无效、adapter layer 不可用或 `supported: false` 都是可恢复的候选失败。`docnav-navigation` 记录候选失败证据后继续遍历后续 adapter。若后续候选成功，selection outcome 是选中 adapter；全部候选失败时，selection outcome 是 format selection failure，并保留候选失败证据。

Static registry 中的 adapter 是 core release 内置 adapter layer 的静态成员。Adapter layer invariant failure 属于 release validation 问题；单次文档操作的 candidate selection 只处理 registry lookup 和 probe outcome。

`ref` 在选定 adapter 内部定位区域。`docnav` core 把非空 ref 原样传给选定 adapter。

## 文档操作执行边界

`docnav-navigation` 的职责是对 navigation command 执行 input resolution、构造内部 operation request，并 dispatch selected adapter operation handler。Adapter implementation source 由 core release 的 static registry 和 linked workspace crate 决定。

Adapter handle 接收的输入已经通过 `docnav-navigation` 的 routing 解析、adapter selection、selected adapter typed-field validation/extraction 和 operation argument binding。Adapter 消费 typed operation input、ref 和 selected adapter native option values；基础类型、required/nullability、allowed value 和范围校验不得留给 handler 消费 raw source value 后再完成。

Adapter operation handler 必须：

- 处理当前 request 指定的一个 operation。
- 为分页操作返回下一页页码，结束时返回 null。
- 按自身声明的 `limit` 预算分页，并始终返回完整 ref。
- 在 outline/find 单条记录超过预算时，保留完整 ref 和最小非空 `label`，并让分页前进；其它 adapter-owned facts 可以省略或压缩。
- 分页文本 `read` content 时，不切断 Unicode 字符。
- 返回结构化 operation result 或 adapter diagnostic。

Operation result 属于已选中 adapter 的执行结果。执行失败、result shape invalid 或 result semantic invalid 是 selected adapter execution failure；candidate selection 在 adapter 选中后结束。
