# CLI 与 MCP 输出

本文是 `docnav` 命令、配置域、输出模式和 MCP 阅读输出映射的主规范。普通 CLI 和 MCP 输出以可读性为主；需要机器稳定解析、兼容校验或自动化断言时，使用 `adapter invoke` 或 `docnav --output protocol-json` 这类完整协议接口。

## `docnav` 核心 CLI

`docnav` 提供所有接入方式共享的核心能力入口：

```text
docnav outline <path> [--adapter <adapter-id>] [--page 1] [--limit-chars 6000] [--output text|readable-json|protocol-json]
docnav read <path> --ref <ref> [--adapter <adapter-id>] [--page 1] [--limit-chars 6000] [--output text|readable-json|protocol-json]
docnav find <path> --query <text> [--adapter <adapter-id>] [--page 1] [--limit-chars 6000] [--output text|readable-json|protocol-json]
docnav info <path> [--adapter <adapter-id>] [--output text|readable-json|protocol-json]
docnav init
docnav doctor
docnav config get|set|unset|list
docnav adapter list
docnav adapter install <source>
docnav adapter update [adapter-id]
docnav adapter remove <adapter-id>
docnav version
```

`docnav` 在启动 adapter `invoke` 前完成：

1. 项目根解析。
2. path 规范化和可访问性检查。
3. adapter 选择：`--adapter` 或 core 简易推断确定一个预选 adapter；预选 probe 失败后逐个 probe 已注册 adapter，并返回第一个成功项。
4. 显式参数、项目配置、用户配置、内置默认值和 manifest 推荐参数合并。
5. page 与 limit_chars 显式化。
6. 输出模式和错误映射选择。

## 配置域

每个可执行 CLI 只读取自己的配置：

| CLI | 项目级配置 | 用户级配置 |
| --- | --- | --- |
| `docnav` | `.docnav/docnav.*` | 用户配置目录中的 `docnav.*` |
| `docnav-markdown` | `.docnav/docnav-markdown.*` | 用户配置目录中的 `docnav-markdown.*` |
| 其他 adapter | `.docnav/<adapter-id>.*` | 用户配置目录中的 `<adapter-id>.*` |
| `docnav-mcp` | `.docnav/docnav-mcp.*` | 用户配置目录中的 `docnav-mcp.*` |

所有 CLI 固定优先级：

```text
显式命令参数
> 项目级 CLI 配置
> 用户级 CLI 配置
> 内置默认值
```

`docnav` 配置拥有 adapter 选择、核心默认参数、输出模式、项目配置、用户配置和管理命令。adapter 配置拥有格式解析参数、格式默认值和 adapter 直接 CLI 输出文本。`docnav-mcp` 配置拥有 MCP TextContent 模板、tool 暴露策略和包装文案。

配置可以控制行为参数、默认阅读文本模板、page 标签、成本说明、guidance、usage、错误建议和 MCP TextContent 包装文本。配置不得改变 protocol-json 字段；readable-json 和 MCP structuredContent 的字段形状用于阅读输出和工具声明校验，不作为完整机器协议。

## 输出模式

### `protocol-json`

用途：完整接口、脚本、调试和兼容性校验；不以可读性为目标。正常阅读不使用该模式。

```text
docnav outline docs/guide.md --output protocol-json
docnav read docs/guide.md --ref "<ref-from-outline>" --output protocol-json
adapter invoke
adapter outline docs/guide.md --output protocol-json
```

文档操作输出完整原始协议 envelope。`manifest` 和 `probe` 输出其专属协议 schema。

`docnav --output protocol-json` 由核心 CLI 生成非空 request id，选择兼容协议版本，解析最终有限参数，再调用 adapter `invoke`。

### 默认阅读文本

用途：人类和 AI 直接阅读。只包含有意义的 ref、display、内容、内容类型、成本、page 状态和必要错误建议。

Markdown adapter outline 文本示例；其中 ref 由 Markdown adapter 生成，核心 CLI 只原样传递：

```text
L1:Guide                     | 9 lines | 0.1 KB
L4:Guide > Install           | 6 lines | 0.1 KB
page: 2
```

### `readable-json`

用途：需要结构化阅读结果但不需要协议 envelope 的 AI 和人类辅助流程。输出不包含 `protocol_version`、`request_id`、`operation`、`ok` 或原始进程错误字段。

`readable-json` 仍属于阅读输出层；字段形状用于文档示例、MCP tool 声明和实现自测，不是脚本长期依赖的机器兼容接口。脚本若需要稳定解析，应使用 `protocol-json` 或 `adapter invoke`。

阅读输出 schema 按 operation 独立定义，见 [schemas](schemas/README.md)。

readable read 保留 adapter 返回的 `content_type`。如果调用方提供 `--adapter <adapter-id>` 或 MCP adapter 参数，`docnav` 先校验该 adapter；失败后再进入 registry 遍历。

阅读错误保留 `code` 和必要 `details` 以便保持阅读语义清晰，同时使用精简、可配置的 error 与 guidance 文本。需要机器可靠错误契约时使用完整协议输出。

## Adapter 管理

`docnav adapter install <source>`、`update [adapter-id]` 和 `remove <adapter-id>` 是正式管理能力，不是占位命令。

首期 `install <source>` 只支持两类来源：

- GitHub 链接：`https://github.com/...` 形式的 adapter 发布链接。`docnav` 必须解析为可执行 adapter 制品，执行 `manifest`，校验 manifest schema 和协议兼容性，并记录来源 URL、解析后的制品信息、manifest 快照和可执行入口。
- 本地可执行文件：指向 adapter exe 的本地路径。`docnav` 必须解析为项目外部或项目内部的绝对可执行路径，执行 `manifest`，校验 manifest schema 和协议兼容性，计算并记录可执行文件 SHA-256 hash。后续运行、`list` 健康状态检查和 `update` 必须重新计算 hash；hash 不一致时不得静默继续使用旧安装记录。

- `list` 输出已安装 adapter、manifest 身份、支持格式、协议范围、安装来源和可用状态。
- `install <source>` 校验失败不得注册；本地可执行文件缺失、不可执行或 hash 无法计算时必须失败。
- `update [adapter-id]` 使用已记录来源获取或重新验证候选版本。GitHub 来源重新解析并获取新制品；本地可执行文件来源重新读取同一路径，重新计算 hash，并在 manifest 和协议兼容性校验通过后更新记录。校验失败时保留旧版本并返回结构化错误。
- `remove <adapter-id>` 注销 adapter 并清理 `docnav` 管理的安装记录；仍被项目配置显式引用时必须失败或给出明确 guidance。

## Adapter 直接 CLI

```text
docnav-markdown outline <path> [--page 1] [--limit-chars 6000] [--max-heading-level 3] [--output text|readable-json|protocol-json]
docnav-markdown read <path> --ref <ref> [--page 1] [--limit-chars 6000] [--output text|readable-json|protocol-json]
docnav-markdown find <path> --query <text> [--page 1] [--limit-chars 6000] [--output text|readable-json|protocol-json]
docnav-markdown info <path> [--output text|readable-json|protocol-json]
docnav-markdown manifest [--output protocol-json]
docnav-markdown probe <path> [--output protocol-json]
```

Markdown 内置默认值：

- outline 每页最多返回 `6000` 字符。
- outline 默认只展示 H1-H3。
- read 每页最多返回 `6000` 字符。
- find 每页最多返回 `6000` 字符。

这些值可由 `docnav-markdown` 自身配置域覆盖。CLI 在执行业务逻辑前解析最终参数；分页操作省略 page 时固定读取第一页，并输出下一页 page 或 null。adapter 文档操作的 `protocol-json` 使用原始协议 envelope；`manifest` 和 `probe` 使用各自专属 schema。

## `docnav-mcp`

`docnav-mcp` 是 Node.js / JavaScript MCP bridge。它通过 stdio 暴露 MCP transport，并依赖系统中可调用的 `docnav` 核心 CLI。

MCP tools：

- `document_outline`
- `document_read`
- `document_find`
- `document_info`

`docnav-mcp` 将 MCP 参数直接映射为核心 `docnav` CLI 调用，将 `docnav` readable 结果转换为 MCP TextContent 和 structuredContent。MCP tools 可传入可选 `adapter` 字符串，映射到 `docnav --adapter <adapter-id>`。它不解析文档内容，不执行格式识别，不管理 adapter，不初始化项目，也不拥有核心配置；adapter 路由和下级适配层调用只由 `docnav` 完成。

MCP 输出属于阅读输出层：

- TextContent 是简洁、可直接阅读的结果，并保留 page 状态。
- structuredContent 使用 operation 对应的精简 readable schema，服务工具声明和客户端展示，不替代完整协议接口。
- structuredContent 不包含完整 invoke envelope。
- TextContent 不复制完整 protocol JSON。
- page 状态使用紧凑文本表达，例如：

```text
page: 2
```

每个 tool 声明精简 readable `outputSchema`。工具声明中的 `outputSchema` 必须内联或随工具声明打包，不依赖远程 schema URL；独立 schema 文件仍作为文档和测试来源。

## 输出文本模板

每个 CLI 的配置域可以控制本 CLI 的输出文本模板，包括：

- 默认阅读文本的标题、分隔符、page 标签和成本说明。
- guidance、usage、错误建议和 MCP TextContent 包装文本。
- 管理命令的人类可读文案。

配置不得改变 protocol-json 的稳定字段、字段类型和错误 code；readable-json 和 MCP structuredContent 的 documented shape 也不得被模板配置任意改写。需要调整提示词或阅读文案时，用户修改配置即可生效，不需要重新编译 CLI。

## 通道与退出码

- 默认阅读文本和 readable JSON 写 stdout。
- `protocol-json` 写 stdout，且只输出一个 JSON 值。
- 诊断写 stderr。
- 未知参数、缺失值和多余参数必须失败。
- `config get` 的 key 不存在时必须返回 `INVALID_REQUEST`。
- 成功退出 `0`；输入错误 `2`；文档/ref/格式错误 `3`；协议或 adapter 进程错误 `4`；内部错误 `1`。

## Page 分页

- 分页操作的 page 是正整数；省略时固定为 `1`，且配置不能改变初始页。
- 结果固定返回下一页 page；null 表示没有更多信息。
- page 非 null 时，调用方保持 path、ref、query、limit_chars 和 options 不变，并把返回的 page 原样用于下一次请求。
- 非 null page 必须等于请求 page 加 1；请求超过末尾时返回空结果和 null。
- CLI、readable JSON 和 MCP 只返回 page 字段表达继续位置。
