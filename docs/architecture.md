# 架构

本文是 Docnav v0 组件职责、输出分层、adapter 选择和进程边界的主规范。

## 核心定位

Docnav 是 CLI-first 的文档导航系统。`docnav` 是核心 CLI，负责识别、路由、分发、管理、配置和项目初始化。调用入口共享 `docnav` CLI 契约，不复制格式识别、adapter 路由或解析逻辑。

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
所有命令先产出成功结果或诊断错误记录，再由输出层统一投影。Document operation 只声明 `readable-view`、`readable-json` 和 `protocol-json` 三种稳定文档输出模式；help、version 和其它非文档命令的成功输出可以保持 PlainText 或命令自有 JSON，但致命诊断仍按当前 output context 走统一错误投影，除非对应 owner 文档明确规定更窄通道。

`docnav` 对文档操作使用单一执行管线：参数归一化、adapter 选择、配置解析、probe、adapter library dispatch 和结果判断不按输出模式分叉。管线产出业务结果、错误通道记录和候选证据；输出层负责按模式序列化、包装并写入 stdout/stderr。

选择机器可读入口表示调用方优先需要稳定、可预测、便于解析的输出；选择阅读入口表示调用方优先需要完成一次可继续的阅读链路。具体 stdout/stderr 通道、JSON shape 和错误包装由 [输出模式](output.md) 与 [原始协议](protocol.md) 定义。

## 组件职责

### `docnav`

负责：

- 提供 `outline`、`read`、`find`、`info`、`init`、`doctor`、`version`、`config` 和 `adapter list`。
- 维护 core release 内置 adapter static registry；`adapter list` 展示该 registry 中的 adapter metadata。
- 提供 `.docnav/` 项目配置和用户级 `docnav` 配置的 `config` 命令入口；配置字段映射、supported key、配置读取和来源合并规则见 [标准参数](standard-parameters.md)。
- 根据 path、项目配置、static registry metadata、`--adapter`、core 简易推断和 probe 选择 adapter。
- 自动选择并调用对应 adapter library handle。
- 在 adapter library dispatch 前消费标准参数机制产出的 core 参数结果，并由 `docnav-navigation` 构造内部 protocol request。
- 统一处理 page、limit、输出模式和错误映射。
- 校验 adapter operation 结果，并转换为默认 readable-view、结构化 readable-json 或完整 protocol 输出。

### 格式 Adapter

负责：

- 使用成熟 parser 识别和解析对应格式。
- 定义格式原生导航参数、源码级 native option registry entries、adapter-side option validation 和内置默认值。
- 生成扁平 outline、ref、业务语义结果和下一页 page。
- 按自身契约解析 ref 并读取。
- 将 readable payload 交给共享 `docnav-readable` 渲染路径，不拥有通用 readable-view 渲染规则。
- 在 manifest 中声明 adapter 身份、支持格式、扩展名、content type 和 capabilities。

adapter 只处理本格式请求，不承担跨格式路由、项目初始化、全局配置管理或调用入口适配。

### 共享库

共享库只抽取稳定契约、机械流程和跨组件重复实现。共享 crate owner：

- `docnav-protocol`：定义原始 protocol request/response、page、错误投影和稳定字段；可提供 JSON decode、protocol field metadata、request id helper，以及 request direct input 与 response/manifest/probe typed contract helper。调用方仍拥有错误归属、field path、diagnostic text、stdout/stderr placement 和 exit behavior。
- `docnav-readable`：提供 readable payload/value helper、仓库内 renderer config、`ReadableViewKind`、readable-view block 渲染器和 conformance vector 类型。readable-view block framing 由本库拥有。
- `docnav-adapter-contracts`：定义 core release 内置 adapter layer 的最小 interface、adapter error、exit category 和共享 operation result contract。格式 adapter 依赖本 crate 暴露 library handle；本 crate 不拥有 parser、ref grammar、routing policy、输出模式或 CLI surface。
- `docnav-navigation`：内部 document operation orchestration owner，负责把 core 标准参数结果构造成 protocol request，并通过 `docnav-adapter-contracts::Adapter` 调用 `outline/read/find/info`。它不拥有 static registry、格式解析、ref 语法或外部 CLI 命令。
- `docnav-json-io`：低层 JSON IO helper，位于 document output 编排下层，只负责 JSON value serialization、newline writing 和 serialization/write failure plumbing；不拥有 schema、protocol/readable wrapper、diagnostic projection、output mode 或 exit code policy。
- `docnav-output`：document operation 输出编排和致命诊断投影 owner，位于 `docnav-readable` 和 `docnav-json-io` 之上、`docnav` core 和 `docnav-navigation` 之下；只承诺 `readable-view`、`readable-json` 和 `protocol-json` 的文档输出形状，help、version、adapter list 或 doctor 的成功输出仍由各命令 owner 定义。
- `docnav-diagnostics`：错误通道 owner，定义 `DiagnosticStack`、`DiagnosticCode`、错误规则、警告规则、`DiagnosticId`、mark 生命周期和 LIFO/drain 语义；详细规则见 [错误通道](diagnostics.md)。本 crate 保存问题记录、机械身份和 code 规则集合，不拥有 surface output format 或 exit code enum。
- `docnav-cli-args`：直接 CLI strict argv token classification owner；输入由调用方提供 command context 和 known value flag metadata。业务参数解析、默认值合并、request 构造和最终 exit behavior 仍由调用方负责；该 crate 不适用于 protocol JSON request decoding。
- `docnav-typed-fields`：字段级事实源 owner，承接 field identity、processing strategy declaration、processing input kind guard、processing build、value kind、字段级 constraints、static default metadata、validation attribution、schema metadata view 和 duplicate identity guard。`FieldDefSet` 聚合通用 typed field definitions，并提供 metadata 与 input-kind guard；input-specific helper 负责把具体输入格式映射到 `FieldDefSet` 的 metadata/validation。当前 JSON helper 承接 JSON path structured path、`serde_json::Value` extraction、unknown-field detection 和 caller processing result。来源合并、CLI argv parsing、operation binding、manifest/probe policy、protocol envelope、readable output、native option handoff policy 和完整 JSON Schema document generation 仍由对应 consumer owner 定义。
- `docnav-standard-parameters`：标准参数解析核心 owner，规则见 [标准参数](standard-parameters.md)。该 crate 消费 `docnav-typed-fields` metadata 和 validation，承接标准参数 registration、source kind/source info、来源合并、默认值、diagnostics、源码级 native option registry handoff 和 operation argument binding metadata；core、`docnav-navigation` 和 adapter layer 的 consumer migration、request construction、adapter-side option validation、输出和错误映射仍由对应 owner 处理。

共享库不定义格式展示字段、格式原生 options 语义、ref 策略、adapter routing、项目配置、process runtime、path display normalization 或跨格式 outline 模型。新增共享 crate 或调整共享库边界时，先同步 owner 文档、schema、examples 和 testing 文档中的边界与验收说明。

## 调用链

通用调用链：

```text
caller
  -> docnav：识别、路由、配置、分页参数和输出模式
  -> docnav-navigation：构造内部 protocol request 并调用 selected adapter library handle
  -> selected adapter layer：解析、导航、生成 ref 和语义结果
  <- protocol result
  <- docnav：转为 CLI 阅读输出或完整协议输出
```

默认文档操作通过当前 core release 编译进来的 workspace adapter crates 和 static registry 选择 adapter implementation source。

## 标准参数边界

标准参数身份、入口字段映射、配置字段映射、来源标记、合并顺序、源码级 native option registry、generic option 合并、adapter option handoff 和标准参数校验由 [标准参数](standard-parameters.md) 定义。架构文档只记录跨组件边界：

- `docnav` 可以消费 core 标准参数结果做 adapter 选择、document context、request planning 和输出模式选择。
- `docnav-navigation` 消费 core 已解析的 operation input，构造内部 protocol request，并调用选定 adapter library handle。
- 显式 public input 默认 strict：未知 argv、多余 positional、当前 operation 不适用的 flag、未映射 request/config 字段和无法归入源码级 native option registry/source 的输入不进入业务执行，入口 owner 必须把它们映射为输入或配置诊断。
- 格式原生 options 只由 core 支持的 public input、配置 `options` object 或对应 registration 声明的 native option source 提供；`docnav` 不从 manifest、core 配置或隐式默认值合成格式专属 options。Adapter selection 后，core 按 selected adapter descriptor 投影支持的 options 并为 unsupported option 返回 native option diagnostic；type mismatch 和 range invalid 由 consuming adapter 返回 adapter-owned structured diagnostic。
- 配置不得改变 protocol envelope、readable JSON 字段或 `DiagnosticCode`；`DiagnosticCode` 由 [错误通道](diagnostics.md) 拥有，protocol/readable 字段由对应 surface owner 文档定义。

## Adapter 选择

`docnav` 对所有文档操作先区分 declared adapter id 和 automatic discovery：

1. 若调用方传入 `--adapter <adapter-id>`，该 id 是 declared adapter id。
2. 若调用方未传入 `--adapter`，项目配置 `defaults.adapter` 优先于用户配置 `defaults.adapter` 作为 declared adapter id。
3. 若存在 declared adapter id，`docnav` 只在 static registry 中查找该 adapter，校验 manifest metadata、capability 并执行 probe 校验。成功则选中，失败则返回 adapter selection diagnostic。
4. 若调用方和配置都未指定 adapter，`docnav` 进入 automatic discovery flow，可以先用 core 简易规则推断候选 adapter id，例如根据 path 扩展名匹配 static registry adapter 的 manifest；无法推断时候选为空。
5. Automatic discovery 中的候选缺失、无法解析、字段不对齐或 probe 失败时，`docnav` 记录候选失败证据并调用 registry 遍历函数。该函数接收已尝试 adapter id 集合，按 registry 顺序跳过已尝试项，返回第一个 probe 成功的 adapter。

所有选择都以 static registry membership 和 adapter probe 结果为准，不能只凭 `--adapter`、配置或扩展名静默选中。Automatic discovery 中候选 adapter 的 metadata 或 probe 契约失败属于可恢复的选择失败：`docnav` 记录候选失败证据并继续遍历，不因单个候选字段缺失、类型不符、语义校验失败或 adapter layer 不可用而直接停止选择流程。`supported: false` 也是普通候选失败证据。

若后续候选成功，前面累积的候选失败只保留为 internal discovery state，成功 document output 不投影这些候选失败。全部候选失败时返回 `FORMAT_UNKNOWN`，primary `DiagnosticRecord.details.candidate_failures` 或 protocol error details 使用候选摘要表达 adapter、阶段和稳定原因码；候选排障细节由 stderr 诊断或内部错误通道按各自契约承载。

显式 `--adapter <adapter-id>` 或配置提供的 adapter id 表达 caller intent。该 adapter 不在 static registry、metadata invalid、probe 失败或 capability 不支持时，`docnav` 返回 adapter selection diagnostic，不把该显式失败伪装成 automatic discovery 成功路径。只有调用方没有显式声明 adapter id 时，候选遍历才是 internal discovery flow。`ref` 只在选定 adapter 内部定位区域，`docnav` 和调用入口只原样传递 ref。

## 项目根与路径

`docnav` 按以下顺序确定项目根：

1. 显式 `--project <path>`。
2. 从启动 cwd 向上查找最近的 `.docnav/`。
3. 未找到时使用启动 cwd。

`docnav` 接受项目根内外的可访问文件路径。相对 path 基于启动 cwd 解析；`document.path` 必须使用 `/`，项目根内路径可以传项目相对路径，项目根外路径传规范化绝对路径。路径不存在、不可读或无法规范化时返回文档路径错误，不能调用 adapter layer。

## 运行边界

- 默认文档操作通过 core release 内置 adapter library handle 执行。
- Adapter layer 只返回 typed operation result 或 adapter error；stdout/stderr、退出码和 readable/protocol 包装由 `docnav` core/output owner 处理。
- 普通 CLI 默认输出 (`readable-view`) 和 `readable-json` 用于阅读；机器校验使用 `docnav --output protocol-json`。
- `doctor` 检查项目/用户配置、static registry 和 adapter layer 可用性。
