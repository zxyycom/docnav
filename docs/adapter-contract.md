# 适配器契约

本文定义格式适配器与 `docnav` core / `docnav-navigation` 的交接契约。它拥有源码级 library interface、静态 descriptor、默认 adapter layer invariant、manifest/probe、adapter 选择规则、operation dispatch、adapter-owned typed-field declarations、adapter-owned ref/result 边界和 selected-adapter typed argument handoff。

## 内置 adapter 接口

每个默认适配器作为 core release 内置 workspace crate 暴露一个 registry-facing adapter definition 或 definition factory，并由 `docnav` static registry 注册。该 definition 是 adapter 作者对 shared layers 的单一 authoring surface：identity、manifest metadata、format descriptors、adapter-owned native option declarations、必需 operation handler handles 和 optional capability groups 都必须从同一个 definition/factory 可达。Adapter-private helper/module 可以拆分 construction，但不得成为 registry、core、CLI、navigation 或 dispatch 的第二个声明入口。

当前实现仍保留 `docnav-adapter-contracts::Adapter` handle 作为受控过渡 dispatch target。过渡层由 `docnav-adapter-contracts` / core registry / `docnav-navigation` 拥有，从 adapter definition 派生当前 handler dispatch path；移除条件是 static registry、CLI native option catalog、adapter inspection、navigation declaration registration、full-read pre-dispatch 和 operation dispatch 都只消费 selected definition 中的 adapter-owned facts。Adapter implementation source 不是 adapter-owned fact，仍由 core static registry 记录。

当前最小 interface 使用 operation handler 粒度：

```text
manifest
probe
declare native option typed fields
execute outline
execute read
execute find
execute info
```

Adapter definition MAY additionally declare a full-read capability group used only by navigation-triggered non-structured full-read outline:

```text
unstructured_full_read content hook
declare full-read cost measurement units
measure full-read cost for requested units
contribute unstructured result facts
```

该 capability group 是可选能力，不替代默认 adapter layer 必需的 `outline`、`read`、`find` 和 `info` handler。`docnav-navigation` 在标准 `outline_mode = "unstructured_full"` 且跳过正常 outline handler 后，才会调用 selected adapter definition 声明的 `unstructured_full_read` content hook。未声明 content hook 时，navigation 可以使用默认 UTF-8 原文读取 fallback；该 fallback 只读取文件、做 UTF-8 decode 并设置基础 `content_type`，不解析 adapter 私有 ref 或格式结构。

Full-read cost measurement declaration SHOULD list the standard cost units the adapter can produce for the non-structured full-read path. Measurement hook MUST receive navigation-selected requested units and return standard `Cost.measurements[]` for the content that full-read would return. 未声明 hook/declaration 时，adapter 的 full-read measurement set 为空。

`manifest` 和 `probe` 是 adapter definition 暴露的 metadata/support facts。默认 adapter layer 的必需文档操作集合为 `outline`、`read`、`find` 和 `info`；进入默认 adapter layer 的 adapter definition 必须全部声明这些 handler。缺少任一 handler 属于 adapter definition invalid 或 release validation 问题，单次 adapter selection 只处理 registry lookup 和 probe outcome。

`docnav-navigation` 接收 core 交出的 raw navigation command、config source descriptors/paths 和 adapter registry，加载 raw project/user config sources，完成 navigation input resolution，构造内部 operation request，并通过 selected adapter definition dispatch 对应 operation。Adapter 返回结构化业务结果或 adapter diagnostic。

格式 adapter 在 definition 中声明格式原生 native options、内置默认值 metadata、adapter-owned option semantics 和 handler binding metadata。`docnav-navigation` 为当前 operation 构造 operation field set：通用 operation 字段由 `docnav-navigation` 声明并注册，selected adapter definition 暴露的 `AdapterOptionSpec` 由使用点注册进同一个 typed-field set。参数汇总边界把同一份 owner-provided facts 投影为 CLI/input metadata 和 config-source metadata；adapter native option 的持久 config source path 由 registry adapter id 与 option key 组合为 `options.<adapter-id>.<option-key>`。解析成功后，navigation 保留 external `OperationArguments.options` 作为 protocol-stable request facts，并额外交付 handler-facing `NativeOptionHandoff`。该 handoff 保留 identity、owner、namespace、key、source、type metadata 和 typed JSON value，供 adapter handler 消费；handler 不再接收 raw CLI argv、raw config JSON 或未校验 native option source value。Input resolution 规则见 [Navigation Input Resolution](navigation-input-resolution.md)。

Adapter 个性化参数使用 `AdapterOptionSpec` 包装 typed-field builder 声明。`path(...)` 声明该 option 注册后进入 `OperationArguments.options` 的 `options.*` 位置；`process(...)` 直接接收 canonical `ProcessStrategy`（`AdapterOptionProcessStrategy` 只是不持有状态的兼容别名）并声明 CLI/config source binding，config source binding 必须使用 adapter-id namespace，例如 `options.docnav-markdown.max_heading_level`。`AdapterOptionSpec` 不保留第二份 source/locator declaration；CLI/config projection 均从底层 typed-field processing metadata 派生。Source path 不作为 final arguments path 的隐含替代；`validation(...)` 和 `default_static(...)` 显式转发到底层 `docnav-typed-fields` 类型。典型形态：

`cli_flag(...)` 必须生成 canonical `CliFlag` locator，`config_path(...)` 必须生成 canonical `ConfigPath` locator；两者分别描述抽取来源，不替代 `path(...)` 定义的 final arguments / handler binding，也不得为同一 source 额外注册兼容 `JsonPath` locator。

```rust
AdapterOptionSpec::builder("docnav.adapters.docnav-markdown.options.max_heading_level")
    .owner("docnav-markdown")
    .operations(&[Operation::Outline, Operation::Find])
    .path(["options", "max_heading_level"])
    .process("cli", AdapterOptionProcessStrategy::cli_flag("--max-heading-level"))
    .process(
        "config",
        AdapterOptionProcessStrategy::config_path(["options", "docnav-markdown", "max_heading_level"]),
    )
    .validation(FieldValidation::int().between(FieldBound::closed(1), FieldBound::closed(6)))
    .default_static(3)
    .build()
```

最小 adapter authoring 形态：

```rust
pub fn markdown_adapter_definition() -> AdapterDefinition<'static> {
    AdapterDefinition::builder("docnav-markdown")
        .adapter(&MarkdownAdapter)
        .manifest(markdown_manifest())
        .required_operation_handlers()
        .native_options(markdown_adapter_options())
        .full_read_capability_group(markdown_full_read_capabilities())
        .build()
        .expect("Markdown adapter definition is valid")
}
```

`markdown_manifest()`、`markdown_adapter_options()` 或其它 private helpers 只服务 adapter-private construction；registry-facing export 仍是一个 definition/factory。

Adapter declarations 必须提供足够的 typed-field facts，让 registry/navigation aggregation 可以在 config-source projection 中校验 `options.<adapter-id>.<option-key>`。这些 facts 至少包括 owner identity、option key、operation applicability、value kind、constraints、static default when declared、source processing metadata 和 operation binding metadata。Adapter handler payload 不随该 config path migration 改变；handler 仍只接收 selected operation 的 typed values 和 `NativeOptionHandoff`。

旧裸 `options.<option-key>` 不是 adapter contract 的兼容输入。Shared layers 不得为它推断 adapter id、迁移值或调用 adapter declaration；它只作为 consuming config/navigation boundary 的普通 unknown/invalid config path 处理。

## 适配器职责

- 使用成熟 parser 解析格式。
- 生成扁平 outline 和 adapter-owned ref。
- 定义格式原生导航参数、adapter-owned native option declarations、内置默认值 metadata、handler binding 和 optional capability groups。
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

格式默认值和 native option declarations 属于 core-linked adapter descriptor 与 adapter-owned typed-field declarations。manifest 保持为 adapter 身份和格式 metadata。

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

存在 declared adapter id 时，`docnav-navigation` 在当前 core release 的 static registry 中查找同名 adapter definition，并执行该 definition 暴露的 probe。registry lookup 失败、probe result 契约无效或 probe 返回 `supported: false` 时，返回 adapter selection diagnostic；probe 返回 `supported: true` 时选中该 adapter definition。声明式选择的通过条件是同名 registry entry 和成功 probe。

不存在 declared adapter id 时，`docnav-navigation` 进入 automatic discovery。Automatic discovery 只按 static registry 顺序遍历 adapter definitions 并执行 probe，返回第一个 `supported: true` 的 adapter definition。

遍历过程中，单个 adapter 的 probe 契约无效、adapter layer 不可用或 `supported: false` 都是可恢复的候选失败。`docnav-navigation` 记录候选失败证据后继续遍历后续 adapter。若后续候选成功，selection outcome 是选中 adapter；全部候选失败时，selection outcome 是 format selection failure，并保留候选失败证据。

Static registry 中的 adapter definition 是 core release 内置 adapter layer 的静态成员。Adapter definition validation failure 属于 release validation 问题；单次文档操作的 candidate selection 只处理 registry lookup 和 probe outcome。

`ref` 在选定 adapter 内部定位区域。`docnav` core 把非空 ref 原样传给选定 adapter。

## 文档操作执行边界

`docnav-navigation` 的职责是对 navigation command 执行 input resolution、构造内部 operation request 和 `NativeOptionHandoff`，并通过 selected adapter definition dispatch selected operation handler。Adapter implementation source 由 core release 的 static registry 和 linked workspace crate 决定。

Adapter handler 接收的输入已经通过 `docnav-navigation` 的 routing 解析、adapter selection、common field declaration、selected adapter definition declaration registration、typed-field validation/extraction、operation argument binding 和 native option handoff construction。Adapter 消费 typed operation input、ref 和 selected adapter native option values；基础类型、required/nullability、allowed value 和范围校验不得留给 handler 消费 raw source value 后再完成。

Adapter operation handler 必须：

- 处理当前 request 指定的一个 operation。
- 为分页操作返回下一页页码，结束时返回 null。
- 按自身声明的 `limit` 预算分页，并始终返回完整 ref。
- 在 outline/find 单条记录超过预算时，保留完整 ref 和最小非空 `label`，并让分页前进；其它 adapter-owned facts 可以省略或压缩。
- 分页文本 `read` content 时，不切断 Unicode 字符。
- 返回结构化 operation result 或 adapter diagnostic。

Operation result 属于已选中 adapter 的执行结果。执行失败、result shape invalid 或 result semantic invalid 是 selected adapter execution failure；candidate selection 在 adapter 选中后结束。

非结构化全文 capability group 只能为 `kind: "unstructured"` outline success result 补充 `content`、`content_type`、`Cost.measurements[]` 或其它稳定 result facts。Hook result MUST NOT 返回 entries、ref、page、continuation 或 readable-only wrapper；readable 文案、block framing 和 cost display 都由输出层从稳定 result facts 派生。
