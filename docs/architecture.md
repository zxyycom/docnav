# 架构

本文是 Docnav v0 制品职责、接入方式、语义层、配置所有权和进程边界的主规范。

## 核心定位

Docnav 是 CLI-first 的文档导航系统。`docnav` 是核心 CLI，负责识别、路由、分发、管理、配置和项目初始化。MCP、skill、AGENTS.md / system prompt 是接入方式，它们共享 `docnav` 契约，不复制解析逻辑。

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
| 阅读输出层 | 为 AI 和人类提供高信息密度结果；不作为长期机器解析接口 | `docnav` 默认输出 (`readable-view`)、`docnav --output readable-json`、MCP 目标输出 |

两层复用相同业务语义，例如 ref、display、内容、成本和 page，但使用不同的传输包装和展示形态。
普通 CLI 和未来 MCP 输出优先服务阅读体验；需要机器稳定解析、兼容校验或自动化断言时，调用完整协议接口。
Document operation 当前只声明 `readable-view`、`readable-json` 和 `protocol-json` 三种输出模式；help、version 和其它非文档纯文本诊断使用独立 PlainText 通道，不参与 document output mode。

`docnav` 对文档操作使用单一执行管线：参数归一化、adapter 选择、配置解析、probe、invoke 和结果判断不按输出模式分叉。管线产出业务结果、稳定错误和候选证据；输出层负责按模式序列化、包装并写入 stdout/stderr。

选择机器可读入口表示调用方优先需要稳定、可预测、便于解析的输出；选择阅读入口表示调用方优先需要完成一次可继续的阅读链路。具体 stdout/stderr 通道、JSON shape 和错误包装由 [输出模式](output.md) 与 [原始协议](protocol.md) 定义。

统一执行管线中的可恢复候选失败不应立即中断整个链路；`docnav` 应跳过失败候选、继续寻找可用 adapter，并把中间失败按顺序保留为候选证据，交由输出层呈现。兜底不能静默吞错；所有被跳过的失败都必须保留 adapter id、阶段和原因。

## 接入层

1. 直接 CLI：人类、脚本和自动化直接调用 `docnav outline/read/find/info`。
2. Skill：通过 skill 指导 agent 使用 `docnav` CLI。
3. AGENTS.md / system prompt：通过项目规则提示 agent 调用 `docnav` CLI。
4. MCP：目标是通过 `docnav-mcp` 将 MCP tool call 映射到 `docnav`；具体 bridge/tools/bin 由 `implement-docnav-mcp-bridge` 交付。

接入层的职责是收集调用者意图、映射参数和展示阅读结果。格式识别、adapter 路由、项目初始化、核心配置和默认参数解析属于 `docnav`。

## 制品职责

### `docnav`

负责：

- 提供 `outline`、`read`、`find`、`info`、`init`、`doctor`、`version`、`config` 和 `adapter list/install/update/remove`。
- 正式执行 adapter 安装、更新、移除和列表管理；首期安装来源为内置 adapter 下载简写和本地可执行文件，安装或更新时必须读取 manifest、校验 manifest schema 和当前协议字段 shape、记录可执行入口，并对本地可执行文件执行 fingerprint 校验。
- 管理 `.docnav/` 项目配置和用户级 `docnav` 配置。
- 根据 path、项目配置、manifest、`--adapter`、core 简易推断和 probe 选择 adapter。
- 自动选择并调用对应 adapter。
- 在启动 `invoke` 前解析显式参数、项目配置、用户配置和 core 内置默认值。
- 统一处理 page、limit_chars、输出模式和错误映射。
- 校验 adapter protocol 结果，并转换为默认 readable-view、结构化 readable-json 或完整 protocol 输出。

### `docnav-mcp`

目标职责：

- 由 `implement-docnav-mcp-bridge` change 使用 Node.js / JavaScript 实现，并定义 npm bin、tool 声明和 bridge wiring。
- 目标通过 stdio 提供 MCP transport。
- 目标暴露 `document_outline`、`document_read`、`document_find`、`document_info`。
- 目标将 MCP tool call 映射为核心 `docnav` CLI 调用。
- 目标将 `docnav` readable 结果转换为 MCP TextContent 和 structuredContent。
- 目标内联或随包打包 MCP tool `outputSchema`。
- 目标依赖系统中可调用的 `docnav` 核心 CLI。
- 不直接调用 adapter，不绕过 `docnav` 的格式识别、配置解析、adapter 选择和错误映射。

当前架构文档只定义 MCP ownership 摘要；完整 handoff 边界见 [MCP Handoff](mcp.md)。JavaScript renderer、TextContent bridge wiring、tool declaration packaging、MCP error mapping 和 MCP 接入层配置键由 `implement-docnav-mcp-bridge` 的 OpenSpec artifacts 定义和验收。

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

- `docnav-protocol`：只定义原始 invoke 协议、page、错误和稳定字段。
- `docnav-readable`：提供 readable payload 到 JSON value 的单一路径、仓库内 renderer config、readable-view 渲染器和 conformance vector 类型。`readable-json` 和 `readable-view` 从同一 typed readable payload 派生。
- `docnav-adapter-sdk`：提供 invoke I/O、协议校验、adapter 直接 CLI 的通用参数解析、命令分发、输出分流、稳定错误映射和通用进程行为。

共享库只承载协议和进程共性，不定义格式展示字段、格式原生 options 语义、ref 策略或跨格式 outline 模型。

## 调用链

通用调用链：

```text
user / agent / skill / prompt / MCP
  -> docnav：识别、路由、配置、分页参数和输出模式
  -> selected adapter invoke：解析、导航、生成 ref 和语义结果
  <- protocol result
  <- docnav：转为 CLI 阅读输出、接入层阅读输出或完整协议输出
```

MCP 目标调用链：

```text
AI Client
  -> docnav-mcp：目标 MCP stdio bridge 和 tool 参数映射
  -> docnav：识别、路由、配置、分页参数和输出模式
  -> selected adapter invoke：解析、导航、生成 ref 和语义结果
  <- protocol result
  <- docnav：readable result
  <- docnav-mcp：目标 TextContent 和 structuredContent
```

每次文档操作启动一个 adapter `invoke` 进程。子进程从 stdin 读取一个完整请求，向 stdout 输出一个原始协议响应，诊断写入 stderr，然后退出。

## 配置所有权

每个可执行 CLI 拥有独立配置域：

| CLI | 配置所有权 |
| --- | --- |
| `docnav` | 当前已实现 `defaults.adapter`、`defaults.limit_chars`、`defaults.output`，以及项目配置、用户配置和管理命令 |
| `docnav-markdown` | Markdown 解析设置、导航默认参数和 adapter 直接 CLI 原生参数默认值；readable-view 渲染路径由共享 `docnav-readable` 和 `docnav-adapter-sdk` 承担 |
| 其他 adapter | 对应格式的解析设置、导航默认参数和直接 CLI 原生参数默认值；readable-view 渲染路径由共享 `docnav-readable` 和 `docnav-adapter-sdk` 承担 |
| `docnav-mcp` | 目标配置域由 `implement-docnav-mcp-bridge` 定义；当前 core 不交付 MCP package 配置键 |

每个 CLI 固定使用：

```text
显式命令参数
> 项目级 CLI 配置
> 用户级 CLI 配置
> 内置默认值
```

配置只在所属 CLI 域内生效。调用方在启动 `invoke` 前必须完成默认参数解析，并在请求中显式传入最终有限参数。格式原生 options 只由 adapter 直接 CLI 或调用方显式参数提供；`docnav` 不从 manifest、配置或隐式默认值合成格式专属 options。

当前已实现配置只控制所属 CLI 明确声明的行为默认值。后续 owner change 如需增加阅读文案配置，只能影响提示文案、guidance、usage 或包装文案；稳定协议字段、readable JSON 字段、MCP structuredContent 字段和错误 code 保持不变。`readable-view` renderer config 只拥有 block 字段声明和 framing 规则，不受用户配置控制。

## Adapter 选择

`docnav` 对所有文档操作先确定一个预选 adapter id，再用统一遍历函数兜底：

1. 若调用方传入 `--adapter <adapter-id>`，该 id 是预选 adapter。
2. 若调用方未传入 `--adapter`，项目配置 `defaults.adapter` 优先于用户配置 `defaults.adapter` 作为预选 adapter。
3. 若调用方和配置都未指定 adapter，`docnav` 使用 core 简易规则推断一个预选 adapter id，例如根据 path 扩展名匹配已注册 adapter 的 manifest；无法推断时预选为空。
4. 若预选 adapter 存在，`docnav` 先解析该 adapter，校验 manifest schema、当前协议字段 shape 并执行 probe 校验。probe 成功则选中，失败时保留失败证据。
5. 若预选 adapter 缺失、无法解析、字段不对齐或 probe 失败，`docnav` 调用 registry 遍历函数。该函数接收已尝试 adapter id 集合，按 registry 顺序跳过已尝试项，返回第一个 probe 成功的 adapter。

所有选择都以 adapter probe 结果为准，不能只凭 `--adapter` 或扩展名静默选中。候选 adapter 的 manifest 或 probe 契约失败属于可恢复的选择失败：`docnav` 记录候选失败证据并继续遍历，不因单个候选字段缺失、类型不符、schema 不匹配、语义校验失败或进程不可用而直接停止选择流程。`supported: false` 也是普通候选失败证据。

若后续候选成功，选择结果必须携带前面累积的候选证据，输出层按 [输出模式](output.md) 的规则呈现为 warning。全部候选失败时返回 `FORMAT_UNKNOWN` 和候选证据。`ref` 只在选定 adapter 内部定位区域，`docnav` 和接入层只原样传递 ref。

## 项目根与路径

`docnav` 按以下顺序确定项目根：

1. 显式 `--project <path>`。
2. 从启动 cwd 向上查找最近的 `.docnav/`。
3. 未找到时使用启动 cwd。

adapter 子进程 cwd 必须设置为项目根；没有可发现项目根时使用启动 cwd。`docnav` 接受项目根内外的可访问文件路径。相对 path 基于启动 cwd 解析；`document.path` 必须使用 `/`，项目根内路径可以传项目相对路径，项目根外路径传规范化绝对路径。路径不存在、不可读或无法规范化时返回文档路径错误，不能启动 adapter。

## 进程边界

- adapter `invoke` 只通过 stdin、stdout 和 stderr 通信。
- `docnav-mcp` 目标 bridge 只通过 stdio 提供 MCP transport；当前实现由 `implement-docnav-mcp-bridge` 交付。
- adapter stdout 只输出该入口的协议或结果。
- 诊断写 stderr。
- 普通 CLI 默认输出 (`readable-view`) 和 `readable-json` 用于阅读；机器校验使用 `protocol-json` 或 `adapter invoke`。
