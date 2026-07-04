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
| 阅读输出 | 为 AI 和人类提供高信息密度结果；不作为长期机器解析接口 | `docnav` 默认输出 (`readable-view`)、`docnav --output readable-json` |

两类输出复用相同业务语义，例如 ref、display、内容、成本和 page，但使用不同的传输包装和展示形态。
普通 CLI 输出优先服务阅读体验；需要机器稳定解析、兼容校验或自动化断言时，调用完整协议接口。
所有命令先产出成功结果或 primary failure，再由输出层统一投影。Document operation 只声明 `readable-view`、`readable-json` 和 `protocol-json` 三种稳定文档输出模式；help、version 和其它非文档命令的成功输出可以保持 PlainText 或命令自有 JSON，但致命诊断仍按当前 output context 走统一错误投影，除非对应 owner 文档明确规定更窄通道。

`docnav` 对文档操作使用单一执行管线：core 完成命令分流、config source descriptor/path handoff 和输出模式识别；`docnav-navigation` 完成 raw config source loading、adapter selection、typed 参数解析、probe、adapter library dispatch 和结果判断。管线不按输出模式分叉；它产出业务结果、primary failure 和候选证据，输出层负责按模式序列化、包装并写入 stdout/stderr。

选择机器可读入口表示调用方优先需要稳定、可预测、便于解析的输出；选择阅读入口表示调用方优先需要完成一次可继续的阅读链路。具体 stdout/stderr 通道、JSON shape 和错误包装由 [输出模式](output.md) 与 [原始协议](protocol.md) 定义。

## 组件职责

### `docnav`

负责：

- 提供 `outline`、`read`、`find`、`info`、`init`、`doctor`、`version`、`config` 和 `adapter list`。
- 维护 core release 内置 adapter static registry；`adapter list` 展示该 registry 中的 adapter metadata。
- 提供 `.docnav/` 项目配置和用户级 `docnav` 配置的 `config` 命令入口；navigation command 的 config source descriptor/path handoff、raw source loading 和参数解析规则见 [Navigation Input Resolution](navigation-input-resolution.md)。
- 解析命令类型；非 navigation 命令由 core 自己处理。
- 对 navigation 命令把 raw command、config source descriptors/paths 和 adapter registry 交给 `docnav-navigation`。
- 统一处理输出模式和错误映射。
- 校验 adapter operation 结果，并转换为默认 readable-view、结构化 readable-json 或完整 protocol 输出。

### 格式 Adapter

负责：

- 使用成熟 parser 识别和解析对应格式。
- 定义格式原生导航参数、adapter-owned typed-field declarations 和内置默认值。
- 生成扁平 outline、ref、业务语义结果和下一页 page。
- 按自身契约解析 ref 并读取。
- 将 readable payload 交给共享 `docnav-readable` 渲染路径，不拥有通用 readable-view 渲染规则。
- 在 manifest 中声明 adapter 身份、支持格式、扩展名、content type 和 adapter layer metadata。

adapter 只处理本格式请求，不承担跨格式路由、项目初始化、全局配置管理或调用入口适配。

### 共享库

共享库只抽取稳定契约、机械流程和跨组件重复实现。共享 crate owner：

- `docnav-protocol`：定义原始 protocol request/response、page、错误投影和稳定字段；可提供 JSON decode、protocol field metadata、request id helper，以及 request direct input 与 response/manifest/probe typed contract helper。调用方仍拥有错误归属、field path、diagnostic text、stdout/stderr placement 和 exit behavior。
- `docnav-readable`：提供 readable payload/value helper、仓库内 renderer config、`ReadableViewKind`、readable-view block 渲染器和 conformance vector 类型。readable-view block framing 由本库拥有。
- `docnav-adapter-contracts`：定义 core release 内置 adapter layer 的最小 interface、adapter error、exit category、adapter-owned native option declaration wrapper 和共享 operation result contract。格式 adapter 依赖本 crate 暴露 library handle；本 crate 不拥有 parser、ref grammar、routing policy、输出模式或 CLI surface，也不拥有 selected adapter declaration 的注册时机。
- `docnav-navigation`：internal document operation orchestration owner，负责 raw project/user config source loading、navigation input resolution、adapter selection、通用字段声明与 selected adapter declarations 的注册合并和解析、`RequestEnvelope` / `OperationArguments` 构造，并通过 `docnav-adapter-contracts::Adapter` 调用 `outline/read/find/info`。它不拥有 static registry 数据源、格式解析、ref 语法、外部 CLI 命令、adapter-owned option 语义或非 navigation 命令行为。
- `docnav-json-io`：低层 JSON IO helper，位于 document output 编排下层，只负责 JSON value serialization、newline writing 和 serialization/write failure plumbing；不拥有 schema、protocol/readable wrapper、diagnostic projection、output mode 或 exit code policy。
- `docnav-output`：document operation 输出编排和致命诊断投影 owner，位于 `docnav-readable` 和 `docnav-json-io` 之上、`docnav` core 和 `docnav-navigation` 之下；只承诺 `readable-view`、`readable-json` 和 `protocol-json` 的文档输出形状，help、version、adapter list 或 doctor 的成功输出仍由各命令 owner 定义。
- `docnav-text-cost`：共享 text cost helper owner，提供只接收纯文本并返回 protocol-compatible `Measurement` 的 `line_cost`、`byte_cost` 和 `token_cost`。调用方拥有文本选择、helper function 集合选择、measurement 顺序、scope 附加、输出暴露和分页预算语义；本 crate 不解析格式、ref、path、operation、adapter policy 或 readable 输出。
- `docnav-diagnostics`：diagnostic/error model primitives helper crate，提供 typed diagnostic code、record draft/record、details validation 和 projection helper materials。它不拥有 operation outcome、surface output format、exit behavior、adapter selection、strict input routing、protocol envelope、readable wrapping 或 CLI surface；这些规则由对应 owner 文档定义。
- `docnav-cli-args`：直接 CLI strict argv token classification owner；输入由调用方提供 command context 和 known value flag metadata。业务参数解析、默认值合并、request 构造和最终 exit behavior 仍由调用方负责；该 crate 不适用于 protocol JSON request decoding。
- `docnav-typed-fields`：字段级事实源 owner，承接 field identity、processing strategy declaration、processing input kind guard、processing build、value kind、字段级 constraints、static default metadata、validation attribution、schema metadata view 和 duplicate identity guard。`FieldDefSet` 聚合通用 typed field definitions，并提供 metadata 与 input-kind guard；input-specific helper 负责把具体输入格式映射到 `FieldDefSet` 的 metadata/validation。当前 JSON helper 承接 JSON path structured path、`serde_json::Value` extraction、unknown-field detection 和 caller processing result。来源合并、CLI argv parsing、operation binding、manifest/probe policy、protocol envelope、readable output、native option handoff policy 和完整 JSON Schema document generation 仍由对应 consumer owner 定义。
- `docnav-parameter-resolution`：当前实现中的参数来源解析 helper。该 crate 消费 `docnav-typed-fields` metadata 和 validation，提供 source kind/source info、来源合并、默认值、diagnostic handoff 和 operation argument binding metadata；长期产品入口按 [Navigation Input Resolution](navigation-input-resolution.md) 描述，`docnav-navigation` 负责加载 navigation config sources、合并通用字段和 selected adapter 注册字段、解析来源并构造 request。

除上述 owner 明确承接的职责外，共享库不定义格式展示字段、格式原生 options 语义、ref 策略、项目配置命令、process runtime、path display normalization 或跨格式 outline 模型。新增共享 crate 或调整共享库边界时，先同步 owner 文档、schema、examples 和 testing 文档中的边界与验收说明。

## 调用链

通用调用链：

```text
caller
  -> docnav：解析命令类型、提供 config source descriptors/paths、处理非 navigation 命令和输出模式
  -> docnav-navigation：加载 raw config sources、解析 routing 输入、选择 adapter、解析 typed 参数、构造内部 protocol request 并调用 selected adapter library handle
  -> selected adapter layer：解析、导航、生成 ref 和语义结果
  <- protocol result
  <- docnav：转为 CLI 阅读输出或完整协议输出
```

默认文档操作通过当前 core release 编译进来的 workspace adapter crates 和 static registry 选择 adapter implementation source。

## 运行边界

- 默认文档操作通过 core release 内置 adapter library handle 执行。
- Adapter layer 只返回 typed operation result 或 adapter error；stdout/stderr、退出码和 readable/protocol 包装由 `docnav` core/output owner 处理。
- 普通 CLI 默认输出 (`readable-view`) 和 `readable-json` 用于阅读；机器校验使用 `docnav --output protocol-json`。
- `doctor` 检查项目/用户配置、static registry 和 adapter layer 可用性。
