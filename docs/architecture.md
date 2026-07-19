# 架构

本文是 Docnav v0 组件职责、输出分层、adapter 选择和进程边界的主规范。

## 核心定位

Docnav 是 CLI-first 的文档导航系统。`docnav` 是核心 CLI，负责命令类型识别、非 navigation 命令、adapter registry 管理、配置管理入口、项目初始化、输出模式和错误投影。Navigation command 的 adapter selection、typed 参数解析、request construction 和 selected adapter dispatch 由 `docnav-navigation` 拥有。调用入口共享 `docnav` CLI 契约，不复制格式识别、adapter selection 或解析逻辑。

核心流程：

```text
outline -> ref -> read
```

`path` 定位文档并供 `docnav` 选择 adapter；`ref` 只定位当前文档内部区域，由 adapter 生成和解析；`page` 表示分页位置；`limit` 表示 adapter-owned numeric budget，具体单位由 adapter owner 文档声明。

## 输出分层

Docnav 文档操作分为两类输出：

| 输出 | 目标 | 入口 |
| --- | --- | --- |
| 原始协议 | 稳定校验、兼容、脚本与调试；不以可读性为目标 | `docnav --output protocol-json` |
| 阅读输出 | 为 AI 和人类提供高信息密度结果；不作为长期机器解析接口 | `docnav` 默认输出或 `docnav --output readable-view` |

两类输出共同消费同一个不可变 `ProtocolResponse`，但使用不同路径：`ProtocolJson` 直接序列化该 response；`Rendered(RenderStrategy)` 把该 response 原样交给 linked code 预先选定的 renderer。`ProtocolResponse` 的字段和业务语义仍由 [原始协议](protocol.md) 拥有，输出层不定义第二套 outcome 或 context。
Runtime invocation log 是第三类独立审计 sink，不属于 document output。它只在 CLI/config 显式启用后写入 owner-documented sink/path；未启用时不得新增 stdout/stderr、protocol/readable 字段、adapter strategy input 或日志文件副作用。启用后，日志事件仍不得进入 `readable-view` 或 `protocol-json` stdout。
普通 CLI 输出优先服务阅读体验；需要机器稳定解析、兼容校验或自动化断言时，调用完整协议接口。
Document operation 在执行 output plan 前把成功或失败统一表示为 `ProtocolResponse`。Public document output 只声明 `readable-view` 和 `protocol-json`：省略 output 或选择 `readable-view` 时，core 构造带内置 renderer 的 `Rendered`；选择 `protocol-json` 时构造 `ProtocolJson`。Help、version 和其它非文档命令的成功输出保持对应 owner 的 PlainText 或命令自有 JSON。

`docnav` 对文档操作使用单一执行管线：core 完成命令分流、config source descriptor/path handoff 和输出模式识别；`docnav-navigation` 完成 raw config source loading、adapter selection、typed 参数解析、probe、adapter library dispatch 和结果判断。Validated structured outline/find base success 还可由 navigation 按 resolved auto-read mode 复用同一 selected adapter 的 read strategy，并在 output plan 前选择 base 或 composed `ProtocolResponse`。管线不按输出模式分叉；正常结果和提前发生的 document failure 都通过既有 protocol projection 形成 `ProtocolResponse`，再由 `ProtocolJson` 或 `Rendered` 执行输出。

选择机器可读入口表示调用方优先需要稳定、可预测、便于解析的输出；选择阅读入口表示调用方优先需要完成一次可继续的阅读链路。具体 stdout/stderr 通道、JSON shape 和错误包装由 [输出模式](output.md) 与 [原始协议](protocol.md) 定义。

## 组件职责

### `docnav`

负责：

- 提供 `outline`、`read`、`find`、`info`、`init`、`doctor`、`version`、`config` 和 `adapter list`。
- 维护 core release 内置 adapter static registry；registry 注册 linked adapter definition factory，`adapter list` 展示 definition 中的 adapter metadata。
- 提供 `.docnav/` 项目配置和用户级 `docnav` 配置的只读 `config inspect` 命令入口；config path flag、`config`/`init`/`doctor` target 语义见 [CLI](cli.md)，navigation command 的 config source descriptor/path handoff、raw source loading 和参数解析规则见 [Navigation Input Resolution](navigation-input-resolution.md)。
- 解析命令类型；非 navigation 命令由 core 自己处理。
- 在一个 core-owned catalog 中声明 release 接受的 document operation 参数及其 source locator、类型、默认值、merge、operation/consumer binding 和可选 exact adapter-id 标记；其中 `docnav.defaults.auto_read` 同时拥有 `--auto-read` 和 `defaults.auto_read` surface。对 navigation 命令把 operation、固定 positional/path facts、normalized document CLI source、config source descriptors/paths、catalog 和 adapter registry 交给 `docnav-navigation`。
- 统一处理输出模式和错误映射。
- 接收并校验 document pipeline 完成的 `ProtocolResponse`，把 navigation 之前发生的 document failure 通过既有 projection 表示为同一类型，并按 CLI mapping 构造 `Rendered` 或 `ProtocolJson`。
- 拥有 runtime invocation logging orchestration：按 [CLI](cli.md) 解析显式启用、日志 sink/path 和可选 content capture root，围绕 navigation-owned adapter selection、request construction、selected adapter dispatch、结果校验和输出投影边界记录 metadata-only JSONL 事件，并保证未启用或日志写入失败时不改变 document operation outcome。

Invocation logging 不把 adapter、protocol envelope 或输出层变成日志 owner。Adapter 继续只返回 typed result 或 diagnostic；`RequestEnvelope` / `ProtocolResponse` 继续由 [原始协议](protocol.md) 拥有；document stdout/stderr 继续由 [输出模式](output.md) 拥有。

### 格式 Adapter

负责：

- 使用成熟 parser 识别和解析对应格式。
- 生成扁平 outline、ref、业务语义结果和下一页 page。
- 按自身契约解析 ref 并读取。
- 消费 core/navigation 已完成来源解析和标准类型 materialization 的 closed operation-specific input；算法正确性需要时可以执行或重复格式语义校验。Auto-read mode 不进入该 input；base outline/find 与 optional nested read 对 adapter 仍是各自独立的单次 operation dispatch。
- 返回 typed operation result 或 adapter error，不选择 output plan，也不拥有通用 readable-view 渲染规则。
- 通过 registry-facing adapter definition/factory 暴露 adapter 身份、manifest/format metadata、固定 strategy 和可选 full-read capabilities/hooks。

adapter 只处理本格式请求，不承担跨格式路由、项目初始化、全局配置管理或调用入口适配。

### 共享库

共享库只抽取稳定契约、机械流程和跨组件重复实现。共享 crate owner：

- `docnav-protocol`：定义原始 protocol request/response、page、错误投影和稳定字段，包括 outline/find success-only optional `auto_read` object；可提供 JSON decode、protocol field metadata、request id helper，以及 request direct input 与 response/manifest/probe typed contract helper。调用方仍拥有错误归属、field path、diagnostic text、stdout/stderr placement 和 exit behavior。
- `docnav-readable`：提供 private readable `Value` 转换、仓库内 renderer config、`ReadableViewKind`、block/framing primitives 和 conformance vector 类型。它只消费调用方准备的 private readable `Value`，不拥有 `ProtocolResponse`、operation/result/error mapping、public output mode 或 stdout/stderr 编排。
- `docnav-adapter-contracts`：定义 core release 内置 adapter layer 的 registry-facing `AdapterDefinition`、definition validation、固定 `Adapter` strategy interface、closed `StandardOperationInput`、adapter error、full-read capabilities/hooks 和共享 operation result contract。格式 adapter 依赖本 crate 暴露 definition/factory 与 strategy；本 crate 不拥有 caller-configurable parameter declaration、source resolution、parser、ref grammar、routing policy、输出模式、CLI surface 或 static registry placement。
- `docnav-navigation`：internal document operation orchestration owner，负责 raw project/user config source loading、full-catalog config validation、adapter selection、selected adapter/current-operation catalog filtering、typed-field resolution、从同一个 `ResolutionResult` 构造 protocol `Options` / closed `StandardOperationInput` / core output projection，并通过 selected adapter definition 调用 `outline/read/find/info` strategy。Validated base response 后的 current-result unique-ref 判定、existing read typed dispatch 和 base/composed response selection 也由本 crate 拥有。它不拥有 parameter catalog authoring、static registry 数据源、格式解析、ref 语法、外部 CLI 命令、格式算法语义或非 navigation 命令行为。
- `docnav-json-io`：低层 JSON IO helper，位于 document output 编排下层，只负责 JSON value serialization、newline writing 和 serialization/write failure plumbing；不拥有 schema、protocol/readable wrapper、diagnostic projection、output mode 或 exit code policy。
- `docnav-output`：document operation 输出编排 owner，位于 `docnav-readable` 和 `docnav-json-io` 之上、`docnav` core 和 `docnav-navigation` 之下；拥有 `ProtocolResponse` 到内置 `RenderStrategy` 的 operation/result/error mapping，包括 successful `auto_read` 到 readable header 与 nested content block 的映射，并从统一 response 执行 `ProtocolJson` 或携带一个 `RenderStrategy` 的 `Rendered`。它负责 renderer invocation 与 document stdout/stderr，help、version、adapter list 或 doctor 的成功输出仍由各命令 owner 定义。
- `docnav-text-cost`：共享 text cost helper owner，提供只接收纯文本并返回 protocol-compatible `Measurement` 的 `line_cost`、`byte_cost` 和 `token_cost`。调用方拥有文本选择、helper function 集合选择、measurement 顺序、scope 附加、输出暴露和分页预算语义；本 crate 不解析格式、ref、path、operation、adapter policy 或 readable 输出。
- `docnav-diagnostics`：diagnostic/error model primitives helper crate，提供 typed diagnostic code、record draft/record、details validation 和 projection helper materials。它不拥有 operation outcome、surface output format、exit behavior、adapter selection、strict input routing、protocol envelope、readable wrapping 或 CLI surface；这些规则由对应 owner 文档定义。
- `docnav-cli-args`：直接 CLI strict argv token classification owner；输入由调用方提供 command context 和 known value flag metadata。业务参数解析、默认值合并、request 构造和最终 exit behavior 仍由调用方负责；该 crate 不适用于 protocol JSON request decoding。
- `docnav-typed-fields`：标准参数定义和字段级事实的唯一 owner。`FieldDef` / `FieldDefSet` 承接 field identity、CLI flag / environment variable / config path processing locator、optional CLI help/value name/Boolean encoding、value kind、字段级 constraints、static default、merge strategy、validation attribution、typed value 和 materialization；definition set build 同时校验 declaration、CLI metadata compatibility 与 processing locator 冲突。来源优先级、具体输入解析、operation binding、public diagnostic、protocol/readable output 和应用 config layout 仍由 consumer owner 定义。
- `cli-config-resolution`：framework-independent resolution core owner，直接消费 canonical `FieldDefSet` 和 normalized `Source` candidates，执行 source priority、field merge、static default fallback、provenance trace、resolution diagnostics、最终 canonical validation 和 all-or-nothing typed materialization。应用 command structure、config layout、adapter/operation/protocol/output 语义仍由 consumer owner 定义。
- `cli-config-resolution-serde`：canonical `FieldDefSet` 的 structured-config input companion，只从 `serde_json::Value` 提取已声明 config paths 的 source candidates。严格拒绝额外 config 字段、字段 applicability 和 public diagnostic mapping 由 consumer owner 决定。Framework-independent resolution core 另保留 declared-only env extractor；只有字段存在 environment locator 时才产生 env candidate。

Parameter aggregation 是上述 existing pieces 对 core catalog canonical field metadata 的协作关系。Core CLI 从 operation-scoped catalog view 生成 public named options并提取 normalized CLI source；navigation 先按 full catalog 校验 config，再按 selected adapter id exact tag 与 current operation 生成 resolution view。未标记 entry 是 common，带标记 entry 只属于同名 static adapter。Fixed positional input 由 navigation 直接映射；project/user config 通过 Serde companion 提取；catalog entry 有 environment locator 时还可提取 env source，并按 `explicit > env > project > user > built_in` 解析，否则保持 `explicit > project > user > built_in`。任何层都不得维护平行 locator table 或复制 field type、constraint、default、validation、merge metadata。

除上述 owner 明确承接的职责外，共享库不定义格式展示字段、格式原生 options 语义、ref 策略、项目配置命令、process runtime、path display normalization 或跨格式 outline 模型。新增共享 crate 或调整共享库边界时，先同步 owner 文档、schema、examples 和 testing 文档中的边界与验收说明。

## 调用链

通用调用链：

```text
caller
  -> docnav：解析命令类型、按 CLI 规则选择 config source descriptors/paths、处理非 navigation 命令和输出模式
  -> docnav-navigation：加载并校验 raw config sources、解析 routing 输入、选择 adapter definition、解析 selected-operation typed 参数，并从一个 resolution result 构造 protocol、closed strategy input 和 core output projection
  -> selected adapter strategy：消费 closed input，解析、导航、执行必要的格式语义校验并生成 ref 和语义结果
  <- typed operation result 或 adapter error
  -> docnav-navigation：校验 base response；符合 current-result unique-ref 条件时，以同一路径、adapter 和 opaque ref 调用 existing read strategy
  <- optional typed read result 或 adapter error
  <- docnav-navigation：选择 validated base 或 success-only composed ProtocolResponse
  <- docnav：navigation 之前的 document failure 通过既有 protocol projection 汇入同一 ProtocolResponse 边界
  -> docnav-output：执行 ProtocolJson 或 Rendered(RenderStrategy)
  <- protocol JSON 或 renderer 返回的完整文本
```

默认文档操作通过当前 core release 编译进来的 workspace adapter crates 和 static registry 选择 linked adapter definition。

### Document named option 派生状态

以下状态只描述 document named option 的实现链路；命令拓扑、固定 positional、config path 与 invocation logging 等 core-owned static surface 不在该派生范围内。

**Current：** caller-configurable document operation 参数已采用单一 core-catalog 路径：

```text
core-owned catalog declaration
  -> operation-scoped CLI field view + normalized candidates
  -> adapter selection
  -> full config validation + selected adapter/current-operation exact-tag view
  -> canonical resolution/materialization
  -> protocol Options + closed StandardOperationInput + core output projection
```

Catalog inventory 是 `page`、`limit`、`pagination.enabled`、`output`、`auto_read` 和 exact-tagged Markdown `max_heading_level`。每项由 core author canonical identity、已启用 locator、标准类型、constraints/default、merge、operation binding 和 closed consumer binding；adapter definition 不声明或发现产品参数。`docnav.defaults.auto_read` 的 CLI/config locator、default、来源顺序和 composition 规则由 [Navigation Input Resolution](navigation-input-resolution.md#unique-ref-auto-read-composition) 单点拥有。Core 的 bounded CLI parser 从 operation-scoped catalog view 生成 arguments/help 与 normalized typed/invalid candidates。

Navigation 使用 routing inputs 选择 adapter，以 full catalog 校验 config shape 和 known adapter namespaces，再以 selected adapter exact tag/current operation view 检查 candidate applicability。Selected candidates 进入 canonical priority、merge、validation 和 materialization；不属于 selected view 的 explicit candidate 或 selected-adapter config value 在 dispatch 前失败。最终 resolution 的 sibling projections 分别提供 protocol `Options`、closed `StandardOperationInput` 和 core-owned output/orchestration facts。`pagination.enabled` 只参与 effective `limit` 归一化；`output` 与 `auto_read` 都不进入 adapter input。

同一 document operation 的 public flags 和 closed consumer targets 保持唯一；catalog construction 在 argv parsing 前拒绝 duplicate/incompatible identity、locator、adapter tag、operation 或 binding。具体 CLI static/generated 分界由 [CLI](cli.md) 拥有；full/selected view、candidate applicability 和 request projection 由 [Navigation Input Resolution](navigation-input-resolution.md) 拥有；closed strategy boundary 由 [适配器契约](adapter-contract.md) 拥有。

Invocation logging 的插桩点跟随同一调用链，但不改变链路输入输出：可记录 core CLI invocation metadata、navigation adapter selection outcome、`RequestEnvelope` 构造状态、selected adapter dispatch outcome、operation/output status、duration、响应大小摘要和 bounded diagnostic summary。日志事件使用 JSON Lines / NDJSON，一行一个独立 JSON event；事件字段 shape 由 [JSON Schema 索引](schemas/json-schema.md) 中的 invocation log schema 校验，字段语义和启用语义仍由本文件、[CLI](cli.md)、[Navigation Input Resolution](navigation-input-resolution.md)、[输出模式](output.md) 和 [原始协议](protocol.md) 分别拥有。

## 运行边界

- 默认文档操作通过 core release 内置 adapter library handle 执行。
- Adapter layer 只返回 typed operation result 或 adapter error；stdout/stderr、退出码和 readable/protocol 包装由 `docnav` core/output owner 处理。
- Auto-read 是 navigation 对既有 single-operation adapter dispatch 的 bounded composition，不新增 adapter operation、adapter mode、protocol request argument 或第二个 output attempt。
- 普通 CLI 默认输出和 `docnav --output readable-view` 使用内置 renderer；机器校验使用 `docnav --output protocol-json`。
- `doctor` 检查项目/用户配置、static registry 和 adapter layer 可用性。
- Runtime invocation logging 是本地运行时审计能力，不是测试/验证日志系统。它不得复用 verify/smoke `.log` 文件或 code-quality observability output 作为运行时 contract；这些开发期 artifact 仍由测试和质量工具 owner 定义。默认实现使用仓库内 JSONL writer；引入外部日志框架前必须完成依赖、初始化行为、feature 选择、输出 sink 隔离和 stdout purity 审计。
