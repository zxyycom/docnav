# 适配器契约

本文定义格式适配器与 `docnav` core / `docnav-navigation` 的交接契约。它拥有源码级 strategy interface、静态 descriptor、默认 adapter layer invariant、manifest/probe、adapter 选择规则、operation dispatch、closed standard input、adapter-owned ref/result 边界和格式语义校验边界。

## 内置 adapter 接口

每个默认适配器作为 core release 内置 workspace crate 暴露一个 registry-facing `AdapterDefinition` factory，并由 `docnav` static registry 注册。Definition 只组合 manifest identity、一个固定 `Adapter` strategy 和可选 `UnstructuredFullReadCapabilities`；它不声明 caller-configurable 参数、source locator、default、merge、validation 或 consumer binding。加载或注册 adapter 本身不能扩大 core 接受的 CLI、env、config 或 protocol input。

当前最小 strategy interface：

```text
manifest
strategy.probe
strategy.outline(OutlineInput)
strategy.read(ReadInput)
strategy.find(FindInput)
strategy.info(InfoInput)
```

`AdapterDefinition` MAY additionally declare full-read capabilities used only by navigation-triggered non-structured full-read outline:

```text
unstructured_full_read content hook
declare full-read cost measurement units
measure full-read cost for requested units
contribute unstructured result facts
```

该 capability 描述是可选能力，不替代固定 strategy 的 `outline`、`read`、`find` 和 `info`。`docnav-navigation` 在标准 `outline_mode = "unstructured_full"` 且跳过正常 outline strategy 后，才会调用 selected adapter definition 暴露的 `unstructured_full_read` hook。未声明 content hook 时，navigation 可以使用默认 UTF-8 原文读取 fallback；该 fallback 只读取文件、做 UTF-8 decode 并设置基础 `content_type`，不解析 adapter 私有 ref 或格式结构。

Full-read cost measurement declaration SHOULD list the standard cost units the adapter can produce for the non-structured full-read path. Measurement hook MUST receive navigation-selected requested units and return standard `Cost.measurements[]` for the content that full-read would return. 未声明 hook/declaration 时，adapter 的 full-read measurement set 为空。

`manifest` 是 definition 暴露的 metadata；`probe` 与四个 operation 方法属于固定 strategy interface，不需要逐 operation method registration 或兼容 dispatch layer。单次 adapter selection 只处理 registry lookup 和 probe outcome。

`docnav-navigation` 接收 core 交出的 fixed command facts、normalized document CLI source、config source descriptors/paths、core parameter catalog 和 adapter registry，完成 source loading、full config validation、adapter selection、selected-operation resolution 与 closed input construction。Definition 只按 `StandardOperationInput` 的 closed variant dispatch 到对应 strategy method；adapter 不接收 raw CLI argv、raw config JSON、parameter declaration、source priority metadata、protocol envelope 或 generic parameter lookup。

最小 adapter definition authoring 形态：

```rust
pub fn markdown_adapter_definition() -> AdapterDefinition<'static> {
    AdapterDefinition::new(
        markdown_manifest(),
        &MarkdownAdapter,
        Some(markdown_full_read_capabilities()),
    )
    .expect("Markdown adapter definition is valid")
}
```

Core parameter catalog 是 caller-configurable 参数的唯一 authoring path；`options.<adapter-id>.<option-key>` 等 source path、exact adapter tag、default 与 binding 都不属于 adapter definition。Input resolution 规则见 [Navigation Input Resolution](navigation-input-resolution.md)。

## 适配器职责

- 使用成熟 parser 解析格式。
- 生成扁平 outline 和 adapter-owned ref。
- 实现固定 operation strategy，并声明可选 full-read capabilities/hooks。
- 消费 closed typed input；必要时执行格式算法所需的语义校验，但不贡献参数声明或 source-resolution facts。
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

Caller-configurable 格式参数及默认值属于 core parameter catalog。Manifest 只保持 adapter 身份和格式 metadata。

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

`docnav-navigation` 的职责是对 navigation command 执行 input resolution、构造 protocol request 与 closed `StandardOperationInput`，并通过 selected adapter definition dispatch selected operation strategy。Adapter implementation 由 core release 的 static registry 和 linked workspace crate 决定。

Adapter strategy 接收的输入已经通过 routing 解析、adapter selection、source priority/merge/default、标准类型 materialization、core-configured validation 和 closed binding。`OutlineInput`、`ReadInput`、`FindInput`、`InfoInput` 只包含对应 operation 的 strategy-visible facts；pagination control、output、raw source、declaration metadata 和 protocol serialized representation 不进入该边界。“Prepared” 不表示所有格式算法语义都已校验，strategy 可以防御性地校验或重复校验 typed value，并以 adapter diagnostic 拒绝不满足格式前置条件的输入。

Adapter operation strategy 必须：

- 处理当前 request 指定的一个 operation。
- 为分页操作返回下一页页码，结束时返回 null。
- 按自身声明的 `limit` 预算分页，并始终返回完整 ref。
- 在 outline/find 单条记录超过预算时，保留完整 ref 和最小非空 `label`，并让分页前进；其它 adapter-owned facts 可以省略或压缩。
- 分页文本 `read` content 时，不切断 Unicode 字符。
- 返回结构化 operation result 或 adapter diagnostic。

Operation result 属于已选中 adapter 的执行结果。执行失败、result shape invalid 或 result semantic invalid 是 selected adapter execution failure；candidate selection 在 adapter 选中后结束。

非结构化全文 hooks 只能为 `kind: "unstructured"` outline success result 补充 `content`、`content_type`、`Cost.measurements[]` 或其它稳定 result facts。Hook result MUST NOT 返回 entries、ref、page、continuation 或 readable-only wrapper；readable 文案、block framing 和 cost display 都由输出层从稳定 result facts 派生。
