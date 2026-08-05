# 适配器契约

本文定义格式适配器与 `docnav` core / `docnav-navigation` 的交接契约。它拥有源码级 factory/document interface、静态 descriptor、默认 adapter layer invariant、manifest pathname hints、adapter 选择规则、operation dispatch、closed standard input、adapter-owned ref/result 边界和格式语义校验边界。

**Current：** manifest pathname hints、no-probe factory、route-before-document-I/O、no-fallback selection、invocation-private `AdapterDocument` 和 ref compatible-view contract 已由代码与测试证明；release artifact 继续证明对外协议和 CLI 兼容性。

## 内置 adapter 接口

每个默认适配器作为 core release 内置 workspace crate 暴露一个 registry-facing `AdapterDefinition` factory function，并由 `docnav` static registry 注册。Definition 只组合 manifest identity、一个 `Adapter` document factory 和可选 `UnstructuredFullReadCapabilities`；它不声明 caller-configurable 参数、source locator、default、merge、validation 或 consumer binding。加载或注册 adapter 本身不能扩大 core 接受的 CLI、env、config 或 protocol input。

新增 adapter 的私有格式策略或安全限制不扩大 `StandardOperationInput`、core parameter catalog 或 source inventory。只有当 caller-configurable fact 在真实异构 adapter 间具有相同 public 语义，并由对应 core owner 接受时，才进入共享参数面；否则由格式 adapter 的固定私有配置拥有。

当前最小共享边界：

```text
AdapterDefinition
  manifest / static capabilities

Adapter
  create one invocation-private AdapterDocument for a normalized path

AdapterDocument
  outline(OutlineInput)
  read(ReadInput)
  find(FindInput)
  info(InfoInput)
  expose declared auxiliary full-read hooks when selected
```

创建 `AdapterDocument` 只建立本次 invocation 的 ownership boundary 并保留 normalized document path；它不得读取目标 metadata、open、read、decode、parse、构造完整 model/index、执行 ref lookup 或触发 document-content diagnostic。每个 document handler 在自身算法第一次确实需要 document access 时才初始化 adapter-private source/model/index view，并保持该 handler 的 semantic-validation-versus-access 顺序。

该边界描述 public operation 与 lifecycle responsibility，不规定 adapter 私有算法的数量、函数拆分或 helper shape。Outward operations 仍是 `outline`、`read`、`find` 和 `info`，但 adapter 可以用任意私有算法实现它们；共享 contract 不要求通用 node/tree、arbitrary lookup、downcast、generic state argument、operation combination registry 或 caller-visible state handle。

`AdapterDefinition` MAY additionally declare full-read capabilities used only by navigation-triggered non-structured full-read outline：

```text
unstructured_full_read content hook
declare full-read cost measurement units
measure full-read cost for requested units
contribute unstructured result facts
```

Capability 描述是可选能力，不替代 `AdapterDocument` 的 `outline`、`read`、`find` 和 `info`。`docnav-navigation` 在标准 `outline_mode = "unstructured_full"` 且跳过正常 outline behavior 后，才会在 selected `AdapterDocument` 上调用声明的 `unstructured_full_read` hook。Selected-adapter cost、content 和 result-facts hooks 复用同一个 invocation-private document view。未声明 content hook 时，navigation 可以使用默认 UTF-8 原文读取 fallback；该 fallback 只读取文件、做 UTF-8 decode 并设置基础 `content_type`，不解析 adapter 私有 ref 或格式结构，也不是 adapter prepared-state reuse 的证明。

Full-read cost measurement declaration SHOULD list the standard cost units the adapter can produce for the non-structured full-read path. Measurement hook MUST receive navigation-selected requested units and return standard `Cost.measurements[]` for the content that full-read would return. 未声明 hook/declaration 时，adapter 的 full-read measurement set 为空。

`manifest` 是 definition 暴露的 metadata；`Adapter` 只创建 document，四个 fixed operation 由 `AdapterDocument` 承接，不需要逐 operation registration 或兼容 dispatch layer。Adapter selection 只使用 registry 与 manifest facts，不执行 adapter-owned detection hook。

任何成功返回 caller-visible ref 的 behavior 都自动承担同一 [Ref 契约](ref-contract.md)。完整的 producer/consumer、兼容视图和 correspondence 规则由该契约及格式 owner 定义；本文只规定 public operation 与 document lifecycle boundary，不按方法名或方法数量定义 ref producer。

`docnav-navigation` 接收 core 交出的 fixed command facts、normalized document CLI source、config source descriptors/paths、core parameter catalog 和 adapter registry，完成 source loading、full config validation、adapter selection、selected-operation resolution 与 closed input construction。Request validation 成功后，navigation 通过 definition 的 factory 为 normalized path 创建至多一个 `AdapterDocument`，再按 `StandardOperationInput` 的 closed variant dispatch 到对应 document behavior。Adapter 不接收 raw CLI argv、raw config JSON、parameter declaration、source priority metadata、protocol envelope、generic parameter lookup 或第二个 parser/state argument。

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

共享 adapter contract 只吸收至少两个真实异构 adapter 已证明相同的职责，不根据预期复用提前抽象。结构遍历和 structured read 是否保留源码顺序属于 adapter-owned、带实现成本的格式策略：adapter 只在格式语义需要且实现证据支持时承诺该顺序；core 和共享 contract 不规定跨格式通用源码顺序，也不要求为此复制文档模型。

- 使用成熟 parser 解析格式。
- 生成扁平 outline 和 adapter-owned ref。
- 实现 fixed operation behavior，并声明可选 full-read capabilities/hooks。
- 消费 closed typed input；必要时执行格式算法所需的语义校验，但不贡献参数声明或 source-resolution facts。
- 返回有限结果和下一页 page。
- 按自身契约解析 ref 并读取，将非法 ref、无匹配 ref 等失败返回为 adapter diagnostic。
- 返回符合 [原始协议](protocol.md#紧凑语义结果) 的紧凑语义结果。

adapter 直接提供本格式的 ref、结构化 item facts、内容、结构化成本、info metadata 和 page。Ref 的共享成功保证见 [Ref 契约](ref-contract.md)；canonical grammar、correspondence、multiplicity、caller-supplied ref 和不兼容视图行为见格式 owner。

## manifest 元数据

Adapter manifest metadata 的字段范围是 adapter 身份和支持格式。稳定字段为：

```text
manifest_version
adapter.id
adapter.name
adapter.version
formats[].id
formats[].extensions[]
formats[].filenames[]
formats[].content_types[]
```

`formats[].id` 是 project-owned normalized format identity。`formats[].extensions[]` 中每个值是带前导点、可包含多个点且不含路径分隔符的完整 basename suffix；它不是只表示最后一个 extension token。`formats[].filenames[]` 中每个值是不含路径分隔符、且不等于 `.` 或 `..` 的 exact basename，数组可以为空。两类 hint 只用于 pathname routing，不证明文档内容符合对应格式 grammar。

Core static registry 必须保证每个 normalized format identity 最多映射一个 adapter definition，同一 ASCII-normalized suffix 或同一 exact filename 在各自 hint kind 内最多映射一个 format identity；同一 hint 的 exact duplicate declaration 也必须拒绝。Registry construction、`doctor` 和 release validation 在 document routing 前阻断这些冲突。不同长度 suffix 的重叠是合法的；exact filename 与 suffix 是不同 hint kind，因此 exact filename 可以覆盖同一 basename 的通用 suffix route。

manifest 字段扩展必须先由本文件和 manifest schema 定义。正式 schema 见 [manifest.schema.json](schemas/manifest.schema.json)。

Caller-configurable 格式参数及默认值属于 core parameter catalog。Manifest 只保持 adapter 身份和格式 metadata。

## Pathname routing hints

**Current：** 本节和下方 adapter 选择规则是 pathname-routing cutover 后的统一契约。

Automatic routing 从 manifest 派生 invocation-private exact-filename、normalized-suffix 和 format-id lookup。Derived indexes、matched hint 与 matched format identity 都是 navigation-private state，不进入 protocol、readable output、invocation log、ref、continuation、typed field 或 adapter operation input。

`Adapter` factory 和 `AdapterDefinition` 不定义 probe method、probe result/reason/version 或 selection-detection hook。内置 adapter 不为 selection 创建 `AdapterDocument`、读取或解析目标文档；协议、schema、decoder、validator、typed-field consumer 和 inspection surface 也不保留 probe compatibility surface。`adapter list` 只投影 manifest identity、format descriptors、capabilities 和 core-owned implementation source。

## adapter 选择

Adapter selection 的输入是 resolved declared adapter id，或 declared adapter id 缺失状态。

Declared adapter id 表达 caller intent。存在 declared adapter id 时，`docnav-navigation` 跳过 pathname routing，只在当前 core release 的 static registry 中做 exact adapter-id lookup。命中即选中该 definition；未命中返回 [原始协议](protocol.md#协议错误对象)定义的 `ADAPTER_UNAVAILABLE` selection diagnostic。Selection success 只证明 linked adapter factory 存在，不证明目标文档有效。

不存在 declared adapter id 时，`docnav-navigation` 使用 core 从 caller path 与 command cwd 词法派生的 routing pathname。该派生和 route lookup 不对目标文档执行 metadata lookup、open、canonicalize、read 或 parse。Automatic lookup 按固定顺序处理 routing pathname 的完整 basename：

1. 先按大小写敏感的 exact spelling 匹配 `filenames[]`。
2. 没有 exact filename 命中时，把完整 basename 与 `extensions[]` suffix 分别做 ASCII 大小写归一化，再执行 end-anchored suffix match。
3. 多个不同长度 suffix 命中时选择字符数最长的声明；例如 `model.schema.JSON` 优先命中 `.schema.json` 而不是 `.json`，`settings.json.backup` 不命中 `.json`。
4. 将命中的 normalized format identity exact lookup 为唯一 adapter definition。

没有 hint 命中时，selection 返回 `FORMAT_UNKNOWN / FORMAT_NOT_RECOGNIZED`，并且不为 path normalization 或 candidate inspection 访问目标文档。Validated registry 不产生 document-level ambiguity 或“format 已识别但 linked adapter 缺失”；若重复 format identity 或同 kind pathname hint 仍逃到 runtime，则按 [原始协议](protocol.md#协议错误对象)的 registry invariant failure 返回。

Selection 命中后，navigation 才进入 core-owned filesystem-backed document path/access normalization，并为 selected operation 构造 normalized document path。Pathname hint 不替代 adapter parse：selected `AdapterDocument` 必须按 operation 正常路径读取、decode、parse 并验证实际文档。Missing/path/encoding、parse、semantic、operation 或 invalid-result failure 都属于已选 definition 的正常结果；navigation 不重新 route、不检查 registry 后续成员，也不 dispatch 第二个 adapter。

`ref` 在选定 adapter 内部定位区域。`docnav` core 把非空 ref 原样传给选定 adapter。

## 文档操作执行边界

`docnav-navigation` 的职责是对 navigation command 执行 input resolution、构造 protocol request 与 closed `StandardOperationInput`，然后通过 selected definition 创建一个 invocation-private `AdapterDocument` 并 dispatch selected operation。Adapter implementation 由 core release 的 static registry 和 linked workspace crate 决定。

`AdapterDocument` 接收的输入已经通过 routing 解析、adapter selection、source priority/merge/default、标准类型 materialization、core-configured validation 和 closed binding。`OutlineInput`、`ReadInput`、`FindInput`、`InfoInput` 只包含对应 operation 的 adapter-visible facts；pagination control、output、raw source、declaration metadata、private state 和 protocol serialized representation 不进入该边界。“Prepared” 不表示所有格式算法语义都已校验，document behavior 可以防御性地校验或重复校验 typed value，并以 adapter diagnostic 拒绝不满足格式前置条件的输入。

Adapter operation behavior 必须：

- 处理当前 request 指定的一个 operation。
- 从 normalized document path 获取并验证该 operation 实际需要的 document view；pathname selection 不构造或传入格式模型。
- 为分页操作返回下一页页码，结束时返回 null。
- 按自身声明的 `limit` 预算分页，并始终返回完整 ref。
- 在 outline/find 单条记录超过预算时，保留完整 ref 和最小非空 `label`，并让分页前进；其它 adapter-owned facts 可以省略或压缩。
- 分页文本 `read` content 时，不切断 Unicode 字符。
- 返回结构化 operation result 或 adapter diagnostic。

Operation result 属于已选中 adapter 的执行结果。执行失败、result shape invalid 或 result semantic invalid 是 selected adapter execution failure；selection 在 adapter 选中后结束，任何这类失败都不得触发 fallback routing。

Request 和 closed-input validation 成功后，navigation 通过 selected definition 创建一个 invocation-private `AdapterDocument` 并 dispatch selected operation。Factory creation 只建立 ownership boundary，不执行 target-document I/O 或 eager preparation；document behavior 接收既有 closed typed input，adapter-private state 不进入该 input，也不作为第二个 parser/state argument 传入。Routing、selection、path/input resolution 或 request construction 失败时不创建 document。

同一 invocation 中第一次需要 document access 的 handler 至多 acquisition/decode/parse 一次兼容的 private view；后续 eligible operation 或 auxiliary hook 复用该 view，不因 stage 变化重新打开 path 或刷新 state。Shared state 只保证 same-view 与 resource reuse，不单独证明 producer/read 一致性；adapter 仍须通过 compatible-view ref contract 和黑盒 conformance evidence。

Private source、parser、model、index、source-region 和 ref-resolution facts 只存在于 `AdapterDocument` 内部。它们不得进入 closed input、operation result、ref、continuation、protocol/readable output、schema、example、invocation log、global registry 或 cross-invocation cache。Pagination request 是新的 invocation，不复用前一页的 document state；独立准备的兼容视图只能依靠 opaque ref 和稳定 adapter contract 恢复相同 selection。

Navigation 在 direct operation、selected-adapter full-read stages 和 eligible unique-ref nested read 结束后立即释放 `AdapterDocument`。Success、adapter diagnostic、validated-base fallback、result validation failure、cancellation 和 unwind 都通过 bounded RAII lifecycle 释放 private state；contract 不定义 public cleanup operation、cleanup result 或 state retention handle。

非结构化全文 hooks 只能为 `kind: "unstructured"` outline success result 补充 `content`、`content_type`、`Cost.measurements[]` 或其它稳定 result facts。参与 selected-adapter execution 的 cost、content 和 result-facts hooks 在同一个 `AdapterDocument` 上运行；hook result MUST NOT 返回 entries、ref、page、continuation 或 readable-only wrapper。Navigation-owned 默认 UTF-8 fallback 仍是没有对应 adapter source capability 时的例外。辅助 hook 不解析、重写或成为 ref identity owner；若未来 hook 发出 ref，它同时成为 ref producer。Readable 文案、block framing 和 cost display 都由输出层从稳定 result facts 派生。
