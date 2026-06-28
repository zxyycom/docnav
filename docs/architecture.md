# 架构

本文是 Docnav v0 制品职责、接入方式、语义层、adapter 选择和进程边界的主规范。

## 核心定位

Docnav 是 CLI-first 的文档导航系统。`docnav` 是核心 CLI，负责识别、路由、分发、管理、配置和项目初始化。Skill 与 AGENTS.md / system prompt 是面向 agent 的使用指引，它们共享 `docnav` 契约，不复制解析逻辑。

核心流程：

```text
outline -> ref -> read
```

`path` 定位文档并供 `docnav` 选择 adapter；`ref` 只定位当前文档内部区域，由 adapter 生成和解析；`page` 表示分页位置；`limit_chars` 表示字符预算。

## 语义层

Docnav 分为两个语义层：

| 层 | 目标 | 入口 |
| --- | --- | --- |
| 原始协议层 | 稳定校验、兼容、脚本与调试；不以可读性为目标 | `adapter invoke`、`docnav --output protocol-json` |
| 阅读输出层 | 为 AI 和人类提供高信息密度结果；不作为长期机器解析接口 | `docnav` 默认输出 (`readable-view`)、`docnav --output readable-json` |

两层复用相同业务语义，例如 ref、display、内容、成本和 page，但使用不同的传输包装和展示形态。
普通 CLI 输出优先服务阅读体验；需要机器稳定解析、兼容校验或自动化断言时，调用完整协议接口。
所有命令先产出成功结果或诊断错误记录，再由输出层统一投影。Document operation 只声明 `readable-view`、`readable-json` 和 `protocol-json` 三种稳定文档输出模式；help、version 和其它非文档命令的成功输出可以保持 PlainText 或命令自有 JSON，但致命诊断仍按当前 output context 走统一错误投影，除非对应 owner 文档明确规定更窄通道。

`docnav` 对文档操作使用单一执行管线：参数归一化、adapter 选择、配置解析、probe、invoke 和结果判断不按输出模式分叉。管线产出业务结果、错误通道记录和候选证据；输出层负责按模式序列化、包装并写入 stdout/stderr。

选择机器可读入口表示调用方优先需要稳定、可预测、便于解析的输出；选择阅读入口表示调用方优先需要完成一次可继续的阅读链路。具体 stdout/stderr 通道、JSON shape 和错误包装由 [输出模式](output.md) 与 [原始协议](protocol.md) 定义。

统一执行管线中的可恢复候选失败不应立即中断整个链路；`docnav` 应跳过失败候选、继续寻找可用 adapter，并把中间失败压入错误通道、保留为候选证据，交由输出层呈现。兜底不能静默吞错；所有被跳过的失败都必须保留 adapter id、阶段和原因。

## 接入层

1. 直接 CLI：人类、脚本和自动化直接调用 `docnav outline/read/find/info`。
2. Skill：通过 skill 指导 agent 使用 `docnav` CLI。
3. AGENTS.md / system prompt：通过项目规则提示 agent 调用 `docnav` CLI。

接入层的职责是收集调用者意图、传递参数并展示阅读结果。格式识别、adapter 路由和项目初始化属于 `docnav`；标准参数映射、配置读取、默认值和来源合并规则见 [标准参数](standard-parameters.md)。

## 制品职责

### `docnav`

负责：

- 提供 `outline`、`read`、`find`、`info`、`init`、`doctor`、`version`、`config` 和 `adapter list/install/update/remove`。
- 正式执行 adapter 安装、更新、移除和列表管理；安装来源包括内置 adapter 下载简写和本地可执行文件，安装或更新时必须读取 manifest、校验 manifest schema 和协议字段 shape、记录可执行入口，并对本地可执行文件执行 fingerprint 校验。
- 提供 `.docnav/` 项目配置和用户级 `docnav` 配置的 `config` 命令入口；配置字段映射、supported key、配置读取和来源合并规则见 [标准参数](standard-parameters.md)。
- 根据 path、项目配置、manifest、`--adapter`、core 简易推断和 probe 选择 adapter。
- 自动选择并调用对应 adapter。
- 在启动 `invoke` 前消费标准参数机制产出的 core 参数结果。
- 统一处理 page、limit_chars、输出模式和错误映射。
- 校验 adapter protocol 结果，并转换为默认 readable-view、结构化 readable-json 或完整 protocol 输出。

### 格式 Adapter

负责：

- 使用成熟 parser 识别和解析对应格式。
- 定义格式原生导航参数、adapter 直接 CLI 原生参数和内置默认值。
- 生成扁平 outline、ref、业务语义结果和下一页 page。
- 按自身契约解析 ref 并读取。
- 将 readable payload 交给共享 `docnav-readable` 渲染路径；adapter 可通过 `docnav-adapter-sdk` 接入该路径，不拥有通用 readable-view 渲染规则。
- 在 manifest 中声明 adapter 身份、支持格式、扩展名、content type 和 capabilities。

adapter 只处理本格式请求，不承担跨格式路由、项目初始化、全局配置管理或接入层适配。

### 共享库

共享库只抽取稳定契约、机械流程和跨制品重复实现。共享 crate owner：

- `docnav-protocol`：定义原始 invoke 协议、page、错误投影和稳定字段；可提供 protocol request/response、manifest 和 probe 的 schema 校验后 decode helper 与 request id helper。调用方仍拥有错误归属、field path、diagnostic text、stdout/stderr placement 和 exit behavior。
- `docnav-readable`：提供 readable payload/value helper、仓库内 renderer config、`ReadableViewKind`、readable-view block 渲染器和 conformance vector 类型。readable-view block framing 由本库拥有。
- `docnav-adapter-sdk`：提供 invoke I/O、协议校验、adapter 直接 CLI 的通用参数解析、命令分发、输出分流、错误输出投影和通用进程行为；可承接 format-neutral paging helper。格式 adapter 仍拥有 parser、ref、display semantics 和格式原生 options。
- `docnav-json-io`：低层 JSON IO helper，位于 document output 编排下层，只负责 JSON value serialization、newline writing 和 serialization/write failure plumbing；不拥有 schema、protocol/readable wrapper、warning、output mode 或 exit code policy。
- `docnav-output`：document operation 输出编排和致命诊断投影 owner，位于 `docnav-readable` 和 `docnav-json-io` 之上、`docnav` core 和 `docnav-adapter-sdk` 之下；只承诺 `readable-view`、`readable-json` 和 `protocol-json` 的文档输出形状，help、version、manifest 或 probe 的成功输出仍由各命令 owner 定义。
- `docnav-diagnostics`：错误通道 owner，定义 `DiagnosticStack`、`DiagnosticCode`、错误规则、警告规则、`DiagnosticId`、mark 生命周期和 LIFO/drain 语义；详细规则见 [错误通道](diagnostics.md)。本 crate 保存问题记录、机械身份和 code 规则集合，不拥有 surface output format 或 exit code enum。
- `docnav-cli-args`：直接 CLI loose argv token scanning owner；输入由调用方提供 command context 和 known value flag metadata。业务参数解析、默认值合并、request 构造和最终 exit behavior 仍由调用方负责；该 crate 不适用于 adapter `invoke` stdin JSON。
- `docnav-typed-fields`：字段级事实源 owner，承接 field identity、processing strategy declaration、processing input kind guard、processing build、value kind、字段级 constraints、static default metadata、validation attribution、schema metadata view 和 duplicate identity guard。`FieldDefSet` 聚合通用 typed field definitions，并提供 metadata 与 input-kind guard；input-specific helper 负责把具体输入格式映射到 `FieldDefSet` 的 metadata/validation。当前 JSON helper 承接 JSON path structured path、`serde_json::Value` extraction、unknown-field detection 和 JSON passthrough。同一 processing id 的 input-specific helper 返回 typed extraction result 和 caller processing result，供标准参数、JSON contract validation 或 schema tooling 消费。来源合并、CLI argv parsing、operation binding、manifest/probe policy、protocol envelope、readable output 和完整 JSON Schema document generation 仍由对应 consumer owner 定义。
- `docnav-standard-parameters`：标准参数解析核心 owner，规则见 [标准参数](standard-parameters.md)。该 crate 消费 `docnav-typed-fields` metadata 和 validation，承接标准参数 registration、source kind/source info、来源合并、默认值、diagnostics、passthrough handoff 和 operation argument binding metadata；core、SDK 和 adapter `invoke` 的 consumer migration、request construction、输出和错误映射仍由对应 owner 处理。

共享库不定义格式展示字段、格式原生 options 语义、ref 策略、adapter routing、项目配置、process runtime、path display normalization 或跨格式 outline 模型。新增共享 crate 或调整共享库边界时，先同步 owner 文档、schema、examples 和 testing 文档中的边界与验收说明。

## 调用链

通用调用链：

```text
user / agent / skill / prompt
  -> docnav：识别、路由、配置、分页参数和输出模式
  -> selected adapter invoke：解析、导航、生成 ref 和语义结果
  <- protocol result
  <- docnav：转为 CLI 阅读输出或完整协议输出
```

每次文档操作启动一个 adapter `invoke` 进程。子进程从 stdin 读取一个完整请求，向 stdout 输出一个原始协议响应，诊断写入 stderr，然后退出。

## 标准参数边界

标准参数身份、入口字段映射、配置字段映射、来源标记、合并顺序、透传和校验由 [标准参数](standard-parameters.md) 定义。架构文档只记录跨制品边界：

- `docnav` 可以消费 core 标准参数结果做 adapter 选择、document context、request planning 和输出模式选择。
- Adapter direct CLI 可以消费 SDK 标准参数结果做 request construction 和 operation build。
- Adapter `invoke` 是独立入口；request `arguments` 是该入口的显式输入，并按 adapter 入口策略处理配置、默认值和未映射字段。
- 格式原生 options 只由 adapter direct CLI、adapter `invoke` request arguments 或对应 registration 声明的来源提供；`docnav` 不从 manifest、配置或隐式默认值合成格式专属 options。
- 配置不得改变 protocol envelope、readable JSON 字段或 `DiagnosticCode`；`DiagnosticCode` 由 [错误通道](diagnostics.md) 拥有，protocol/readable 字段由对应 surface owner 文档定义。

## Adapter 选择

`docnav` 对所有文档操作先确定一个预选 adapter id，再用统一遍历函数兜底：

1. 若调用方传入 `--adapter <adapter-id>`，该 id 是预选 adapter。
2. 若调用方未传入 `--adapter`，项目配置 `defaults.adapter` 优先于用户配置 `defaults.adapter` 作为预选 adapter。
3. 若调用方和配置都未指定 adapter，`docnav` 使用 core 简易规则推断一个预选 adapter id，例如根据 path 扩展名匹配已注册 adapter 的 manifest；无法推断时预选为空。
4. 若预选 adapter 存在，`docnav` 先解析该 adapter，校验 manifest schema、协议字段 shape 并执行 probe 校验。probe 成功则选中，失败时保留失败证据。
5. 若预选 adapter 缺失、无法解析、字段不对齐或 probe 失败，`docnav` 调用 registry 遍历函数。该函数接收已尝试 adapter id 集合，按 registry 顺序跳过已尝试项，返回第一个 probe 成功的 adapter。

所有选择都以 adapter probe 结果为准，不能只凭 `--adapter` 或扩展名静默选中。候选 adapter 的 manifest 或 probe 契约失败属于可恢复的选择失败：`docnav` 记录候选失败证据并继续遍历，不因单个候选字段缺失、类型不符、schema 不匹配、语义校验失败或进程不可用而直接停止选择流程。`supported: false` 也是普通候选失败证据。

若后续候选成功，选择结果必须携带前面累积的候选证据，输出层按 [输出模式](output.md) 的规则呈现为 warning。全部候选失败时返回 `FORMAT_UNKNOWN` 和候选证据。`ref` 只在选定 adapter 内部定位区域，`docnav` 和接入层只原样传递 ref。

## 项目根与路径

`docnav` 按以下顺序确定项目根：

1. 显式 `--project <path>`。
2. 从启动 cwd 向上查找最近的 `.docnav/`。
3. 未找到时使用启动 cwd。

adapter 子进程 cwd 必须设置为项目根；没有可发现项目根时使用启动 cwd。`docnav` 接受项目根内外的可访问文件路径。相对 path 基于启动 cwd 解析；`document.path` 必须使用 `/`，项目根内路径可以传项目相对路径，项目根外路径传规范化绝对路径。路径不存在、不可读或无法规范化时返回文档路径错误，不能启动 adapter。

Adapter direct CLI 自行执行时，不复用 core `--project` 参数，也不使用 document path 发现配置项目根。Adapter direct CLI 配置项目根、默认配置文件名、覆盖参数和配置源失败规则见 [标准参数](standard-parameters.md#输入与配置映射)。

## 进程边界

- adapter `invoke` 只通过 stdin、stdout 和 stderr 通信。
- Adapter `invoke` 是独立入口；request `arguments` 是该入口的显式输入，配置读取、来源合并和未映射字段策略见 [标准参数](standard-parameters.md)。
- adapter stdout 只输出该入口的协议或结果。
- 诊断写 stderr。
- 普通 CLI 默认输出 (`readable-view`) 和 `readable-json` 用于阅读；机器校验使用 `protocol-json` 或 `adapter invoke`。
