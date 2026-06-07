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
| 阅读输出层 | 为 AI 和人类提供高信息密度结果；不作为长期机器解析接口 | `docnav` 默认输出、`docnav --output readable-json`、MCP |

两层复用相同业务语义，例如 ref、display、内容、成本和 page，但使用不同的传输包装和展示形态。
普通 CLI 和 MCP 输出优先服务阅读体验；需要机器稳定解析、兼容校验或自动化断言时，调用完整协议接口。

## 接入层

1. 直接 CLI：人类、脚本和自动化直接调用 `docnav outline/read/find/info`。
2. Skill：通过 skill 指导 agent 使用 `docnav` CLI。
3. AGENTS.md / system prompt：通过项目规则提示 agent 调用 `docnav` CLI。
4. MCP：通过 `docnav-mcp` 将 MCP tool call 映射到 `docnav`。

接入层的职责是收集调用者意图、映射参数和展示阅读结果。格式识别、adapter 路由、项目初始化、核心配置和默认参数解析属于 `docnav`。

## 制品职责

### `docnav`

负责：

- 提供 `outline`、`read`、`find`、`info`、`init`、`doctor`、`version`、`config` 和 `adapter list/install/update/remove`。
- 正式执行 adapter 安装、更新、移除和列表管理；首期安装来源为 GitHub 链接和本地可执行文件，安装或更新时必须读取 manifest、校验 manifest schema 和当前协议字段 shape、记录可执行入口，并对本地可执行文件执行 hash 校验。
- 管理 `.docnav/` 项目配置和用户级 `docnav` 配置。
- 根据 path、项目配置、manifest、`--adapter`、core 简易推断和 probe 选择 adapter。
- 自动选择并调用对应 adapter。
- 在启动 `invoke` 前解析显式参数、项目配置、用户配置、内置默认值和 manifest 推荐参数。
- 统一处理 page、limit_chars、输出模式和错误映射。
- 校验 adapter protocol 结果，并转换为默认阅读文本、结构化阅读输出或完整 protocol 输出。

### `docnav-mcp`

负责：

- 使用 Node.js / JavaScript 实现，并作为 npm 可安装 bin 暴露，例如 `docnav-mcp`。
- 通过 stdio 提供 MCP transport。
- 暴露 `document_outline`、`document_read`、`document_find`、`document_info`。
- 将 MCP tool call 映射为核心 `docnav` CLI 调用。
- 将 `docnav` 结果转换为 MCP TextContent 和 structuredContent。
- 内联或随包打包 MCP tool `outputSchema`。
- 读取自身 MCP 接入层配置，例如 TextContent 模板、tool 暴露策略和包装文案。
- 依赖系统中可调用的 `docnav` 核心 CLI。
- 不直接调用 adapter，不绕过 `docnav` 的格式识别、配置解析、adapter 选择和错误映射。

### 格式 Adapter

负责：

- 使用成熟 parser 识别和解析对应格式。
- 定义格式原生导航参数和内置默认值。
- 生成扁平 outline、ref、语义结果和下一页 page。
- 解析 ref 的格式定位部分并唯一读取。
- 定义 adapter 直接 CLI 的阅读文本和 readable JSON。
- 在 manifest 中声明格式原生推荐参数，使 `docnav` 能在 invoke 前显式化最终参数。

adapter 只处理本格式请求，不承担跨格式路由、项目初始化、全局配置管理或接入层适配。

### 共享库

- `docnav-protocol`：只定义原始 invoke 协议、page、错误和稳定字段。
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

MCP 专属调用链：

```text
AI Client
  -> docnav-mcp：MCP stdio bridge 和 tool 参数映射
  -> docnav：识别、路由、配置、分页参数和输出模式
  -> selected adapter invoke：解析、导航、生成 ref 和语义结果
  <- protocol result
  <- docnav：readable result
  <- docnav-mcp：TextContent 和 structuredContent
```

每次文档操作启动一个 adapter `invoke` 进程。子进程从 stdin 读取一个完整请求，向 stdout 输出一个原始协议响应，诊断写入 stderr，然后退出。

## 配置所有权

每个可执行 CLI 拥有独立配置域：

| CLI | 配置所有权 |
| --- | --- |
| `docnav` | adapter 选择、核心默认参数、输出模式、项目配置、用户配置和管理命令 |
| `docnav-markdown` | Markdown 导航默认参数、解析设置、直接 CLI 输出文本 |
| 其他 adapter | 对应格式的默认参数、解析设置和直接 CLI 输出文本 |
| `docnav-mcp` | MCP TextContent 模板、tool 暴露策略和 MCP 包装文案 |

每个 CLI 固定使用：

```text
显式命令参数
> 项目级 CLI 配置
> 用户级 CLI 配置
> 内置默认值
```

配置只在所属 CLI 域内生效。调用方在启动 `invoke` 前必须完成默认参数解析，并在请求中显式传入最终有限参数。格式 adapter 通过 manifest 声明格式原生推荐参数；`docnav` 可以选择并原样传入 adapter。

每个 CLI 可以通过自身配置域调整输出文本模板、guidance、usage 和错误建议。配置只能影响阅读文本和提示文案，稳定协议字段、readable JSON 字段、MCP structuredContent 字段和错误 code 保持不变。

## Adapter 选择

`docnav` 对所有文档操作先确定一个预选 adapter id，再用统一遍历函数兜底：

1. 若调用方传入 `--adapter <adapter-id>`，该 id 是预选 adapter。
2. 若调用方未传入 `--adapter`，项目配置 `defaults.adapter` 优先于用户配置 `defaults.adapter` 作为预选 adapter。
3. 若调用方和配置都未指定 adapter，`docnav` 使用 core 简易规则推断一个预选 adapter id，例如根据 path 扩展名匹配已注册 adapter 的 manifest；无法推断时预选为空。
4. 若预选 adapter 存在，`docnav` 先解析该 adapter，校验 manifest schema、当前协议字段 shape 并执行 probe 校验。probe 成功则选中，失败时保留失败证据。
5. 若预选 adapter 缺失、无法解析、字段不对齐或 probe 失败，`docnav` 调用 registry 遍历函数。该函数接收已尝试 adapter id 集合，按 registry 顺序跳过已尝试项，返回第一个 probe 成功的 adapter。

所有选择都以 adapter probe 结果为准，不能只凭 `--adapter` 或扩展名静默选中。全部候选失败时返回 `FORMAT_UNKNOWN` 和候选证据。`ref` 只在选定 adapter 内部定位区域，`docnav` 和接入层只原样传递 ref。

## 项目根与路径

`docnav` 按以下顺序确定项目根：

1. 显式 `--project <path>`。
2. 从启动 cwd 向上查找最近的 `.docnav/`。
3. 未找到时使用启动 cwd。

adapter 子进程 cwd 必须设置为项目根；没有可发现项目根时使用启动 cwd。`docnav` 接受项目根内外的可访问文件路径。相对 path 基于启动 cwd 解析；`document.path` 必须使用 `/`，项目根内路径可以传项目相对路径，项目根外路径传规范化绝对路径。路径不存在、不可读或无法规范化时返回文档路径错误，不能启动 adapter。

## 进程边界

- adapter `invoke` 只通过 stdin、stdout 和 stderr 通信。
- `docnav-mcp` 只通过 stdio 提供 MCP transport。
- adapter stdout 只输出该入口的协议或结果。
- 诊断写 stderr。
- 普通 CLI 默认输出和 `readable-json` 用于阅读；机器校验使用 `protocol-json` 或 `adapter invoke`。
