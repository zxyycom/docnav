# CLI 与 MCP 输出

本文是 `docnav` 命令、配置域、输出模式和 MCP 阅读输出映射的主规范。普通 CLI 和 MCP 输出以可读性为主；需要机器稳定解析、兼容校验或自动化断言时，使用 `adapter invoke` 或 `docnav --output protocol-json` 这类完整协议接口。所有 JSON 输出都必须保持 documented shape 和明确通道归属；`protocol-json` 是完整机器协议，`readable-json` 是结构化阅读输出。

## `docnav` 核心 CLI

`docnav` 提供所有接入方式共享的核心能力入口：

```text
docnav outline <path> [--adapter <adapter-id>] [--page 1] [--limit-chars 6000] [--output text|readable-json|protocol-json]
docnav read <path> --ref <ref> [--adapter <adapter-id>] [--page 1] [--limit-chars 6000] [--output text|readable-json|protocol-json]
docnav find <path> --query <text> [--adapter <adapter-id>] [--page 1] [--limit-chars 6000] [--output text|readable-json|protocol-json]
docnav info <path> [--adapter <adapter-id>] [--output text|readable-json|protocol-json]
docnav init
docnav doctor
docnav config get|set|unset|list [--user] [--path <path>] [--operation outline|read|find|info]
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
4. 显式参数、项目配置、用户配置和 core 内置默认值合并。
5. page 与 limit_chars 显式化。
6. 输出模式和错误映射选择。

Rust CLI argv 结构解析以 `clap` 或 `clap` builder API 为基础。`clap` 承载本 change 覆盖入口的命令、子命令、固定参数、默认值、枚举值和 help；Docnav 在 command/operation 确定后只对当前 operation 实际使用的参数做类型、范围和枚举校验。成功解析的 document CLI argv 先进入 canonical document operation input 或等价 semantic request，再进入 adapter routing、invoke request 构造和 output dispatch。该内部语义输入不是 protocol envelope、schema 稳定类型或 MCP 传输模型。

未知 argv、多余 positional 和当前 operation 不使用的已知参数不阻断有效文档操作；它们只生成 warning metadata。当前 operation 实际使用的参数仍严格校验：缺少必需 path/ref/query、非法 page、非法 limit_chars、非法 output 或非法 native option 必须返回输入错误。当前 operation 不使用的 known 参数即使值不符合其它 operation 的类型、范围或枚举规则，也不得触发 eager typed failure。

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

`docnav config set` 和 `unset` 默认写项目配置；传入 `--user` 时写用户配置。`config list` 不带 path 时列出 core 配置域的当前生效值；`config list --path <path> [--operation outline|read|find|info]` 解析文档上下文，展示该 path 触发的 adapter、core 参数来源和最终默认参数。

## 输出模式

`--output` 只选择输出层的序列化、错误包装和通道承载方式，不改变 `docnav` 的 adapter 选择、配置合并、参数显式化、probe、invoke 或业务结果判断。实现应先产出统一 outcome，再按输出模式渲染为文本、readable JSON、MCP structuredContent 或 protocol envelope。

机器可读输出必须优先保持稳定和可解析。若调用方选择 `protocol-json` 或 `readable-json`，stdout 必须只输出一个符合该模式 documented shape 的 JSON 值；错误发生在 CLI 参数解析、adapter 选择、adapter invoke 或输出转换阶段时，只要输出模式可以从 argv 或请求中确定，也必须使用对应 JSON 错误形态。无法确定 operation 时，协议错误 envelope 使用 `operation: null`。

统一执行管线按 [架构](architecture.md#adapter-选择) 累积可恢复候选失败；本文件只定义这些候选证据在各输出模式中如何承载为 warning。

### `protocol-json`

用途：完整接口、脚本、调试和兼容性校验；不以可读性为目标。正常阅读不使用该模式。

```text
docnav outline docs/guide.md --output protocol-json
docnav read docs/guide.md --ref "<ref-from-outline>" --output protocol-json
adapter invoke
adapter outline docs/guide.md --output protocol-json
```

文档操作输出完整原始协议 envelope。`manifest` 和 `probe` 输出其专属协议 schema。

`docnav --output protocol-json` 由核心 CLI 生成非空 request id，按当前协议 schema 和字段 shape 解析最终有限参数，再调用 adapter `invoke`。

`protocol-json` stdout 不承载直接 CLI 兼容性 warning 或 adapter 选择候选 warning。若直接 CLI argv 中存在被兼容忽略的 token，或 adapter 选择过程中跳过了不可用、契约不匹配、probe 不支持的候选，warning 写入 stderr，stdout 仍只输出一个符合 protocol response schema 的 JSON envelope。若参数解析失败但 argv 已能确定 `--output protocol-json`，stdout 仍输出 protocol failure envelope，而不是退回文本错误。

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

`readable-json` 仍属于阅读输出层中的结构化机器友好形态。它必须保持 documented shape，便于 AI、工具和轻量自动化解析阅读结果；但它不包含完整协议 envelope，也不替代 `protocol-json` 或 `adapter invoke` 的完整机器兼容接口。脚本若需要跨版本稳定错误 envelope、request id 或协议兼容校验，应使用 `protocol-json` 或 `adapter invoke`。

阅读输出 schema 按 operation 独立定义，见 [schemas](schemas/README.md)。

成功结果存在直接 CLI 兼容性 warning 时，`readable-json` 必须在顶层输出 `warnings` 数组；没有 warning 时省略该字段。每个 warning item 必须使用稳定 warning envelope：`id`、非空 `reason`、稳定 `effect` 和 `details` 对象。CLI argv warning 使用 `id: "cli_argv_ignored"`，相关 argv token 只能作为 `details.tokens` 等 family-specific detail 表达。CLI argv warning 的 exact token 分组、`reason` 文案和 token 消费顺序不作为稳定契约。

成功结果存在 adapter 选择候选 warning 时，`readable-json` 同样必须在顶层 `warnings` 数组中保留。adapter candidate warning 使用 `id: "adapter_candidate_failure"`，`effect: "candidate_skipped"`，并在 `details` 中保留 `adapter_id`、`stage`、`code` 和可选 `preselected`。没有 warning 时省略该字段。

readable read 保留 adapter 返回的 `content_type`。如果调用方提供 `--adapter <adapter-id>` 或 MCP adapter 参数，`docnav` 先校验该 adapter；失败后再进入 registry 遍历。预选 adapter 失败不直接中断阅读链路，而是作为候选 warning 保留。

阅读错误保留 `code` 和必要 `details` 以便保持阅读语义清晰，同时使用精简、可配置的 error 与 guidance 文本。需要机器可靠错误契约时使用完整协议输出。

## Adapter 管理

`docnav adapter install <source>`、`update [adapter-id]` 和 `remove <adapter-id>` 是正式管理能力，不是占位命令。

首期 `install <source>` 只支持两类来源：

- 内置 adapter 下载简写：例如 `markdown`。`docnav` 必须解析为可执行 adapter 制品，执行 `manifest`，校验 manifest schema、必需字段和当前协议字段 shape，并记录 source key、解析后的制品信息、manifest 快照和可执行入口。
- 本地可执行文件：指向 adapter exe 的本地路径。`docnav` 必须解析为项目外部或项目内部的绝对可执行路径，执行 `manifest`，校验 manifest schema、必需字段和当前协议字段 shape，计算并记录可执行文件 SHA-256 fingerprint。普通文档操作不为 fingerprint 校验读取整个 adapter 可执行文件；install、update 和显式健康检查必须重新计算 fingerprint，fingerprint 不一致时不得静默继续使用旧安装记录。

- `list` 输出已安装 adapter、manifest 身份、支持格式、安装来源和可用状态。
- `install <source>` 校验失败不得注册；本地可执行文件缺失、不可执行或 fingerprint 无法计算时必须失败。
- `update [adapter-id]` 使用已记录来源获取或重新验证候选制品。内置下载来源重新走内置映射；本地可执行文件来源重新读取同一路径，重新计算 fingerprint，并在 manifest schema 和协议字段 shape 校验通过后更新记录。校验失败时保留旧记录并返回结构化错误。
- `remove <adapter-id>` 注销 adapter 并清理 `docnav` 管理的安装记录；仍被项目配置显式引用时必须失败或给出明确 guidance。

## Adapter 直接 CLI

```text
docnav-markdown outline <path> [--page 1] [--limit-chars 6000] [--max-heading-level 3] [--output text|readable-json|protocol-json]
docnav-markdown read <path> --ref <ref> [--page 1] [--limit-chars 6000] [--output text|readable-json|protocol-json]
docnav-markdown find <path> --query <text> [--page 1] [--limit-chars 6000] [--output text|readable-json|protocol-json]
docnav-markdown info <path> [--output text|readable-json|protocol-json]
docnav-markdown manifest [--output protocol-json]
docnav-markdown probe <path> [--output protocol-json]
docnav-markdown invoke
```

Markdown 内置默认值：

- outline 每页最多返回 `6000` 字符。
- outline 默认只展示 H1-H3。
- read 每页最多返回 `6000` 字符。
- find 每页最多返回 `6000` 字符。

这些值可由 `docnav-markdown` 自身配置域覆盖。CLI 在执行业务逻辑前解析最终参数；分页操作省略 page 时固定读取第一页，并输出下一页 page 或 null。adapter 文档操作的 `protocol-json` 使用原始协议 envelope；`manifest` 和 `probe` 使用各自专属 schema。

### 直接 CLI 兼容参数规则

所有 Docnav 直接 CLI argv 使用同一兼容规则：`clap` 或 `clap` builder API 承载固定命令、已知参数、默认值、枚举值和 help；Docnav 或 SDK 在确定 command/operation 后只校验当前 operation 实际使用的参数。未知 flag、多余 positional、当前 operation 不使用的已知 flag 写 warning 后忽略；warning 使用稳定 envelope，并将相关原始 token 放入 `details.tokens`。`--unknown=value` 可作为一个未知 flag token 记录；`--unknown value` 的具体 token 分组和消费顺序不是稳定契约。当前 operation 不使用的已知有值 flag 只按 flag 形状消费并写 warning，不校验该 value 的业务合法性。已知必需参数缺失、当前 operation 实际使用的已知 flag 缺少值或值非法必须失败。

兼容规则只适用于直接 CLI argv，不适用于 adapter `invoke` stdin JSON。`invoke` 请求仍按 protocol request schema 严格校验，未知字段或参数类型错误不得被兼容忽略。

## 命令族矩阵

| 命令族 | Owner | 文档 semantic request | 宽松 argv | ignored argv 诊断通道 | protocol-shaped stdout 边界 | help 是否执行业务 | 本 change 验收 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Core document operations：`docnav outline/read/find/info` | `docnav` core CLI | 是 | 是；只宽松 unknown、extra 和 unused known | text/readable-json 承载 warning；protocol-json 写 stderr | `--output protocol-json` stdout 只含 protocol response envelope | 否 | 是 |
| Core non-document commands：`config/init/doctor/version` | `docnav` core CLI | 否 | 类型化命令；可对无关 argv 给诊断 | 当前命令输出层或 stderr | 不产生 document protocol response | 否 | 代表性验收 |
| Core adapter management：`docnav adapter list/install/update/remove` | `implement-docnav-adapter-management` change | 否 | 由 adapter management owner 定义 | 由 adapter management owner 定义 | 管理命令 schema/通道由 owner 定义 | 否 | 否，只记录边界 |
| Adapter direct document operations：`docnav-markdown outline/read/find/info` | adapter SDK + format adapter | 是 | 是；只宽松 unknown、extra 和 unused known/native | text/readable-json 承载 warning；protocol-json 写 stderr | `--output protocol-json` stdout 只含 protocol response envelope | 否 | 是 |
| Adapter direct machine commands：`manifest/probe/invoke` | adapter SDK | `invoke` valid request 进入等价 semantic request；manifest/probe 否 | manifest/probe CLI argv 可宽松；invoke JSON 严格 | manifest/probe warning 写 stderr；invoke transport error 写 protocol failure | stdout 只含 manifest、probe 或 protocol response schema payload | 否 | 是 |
| Help commands：root help 和子命令 help | 各 CLI owner | 否 | 不适用 | stdout/stderr 只输出 help 文本 | 不输出 protocol/readable payload | 否 | 是 |
| MCP bridge：`document_outline/read/find/info` | `docnav-mcp` | 否，映射为核心 `docnav` CLI | MCP arguments 不按 argv 宽松处理 | MCP structuredContent 复用 readable warning envelope | 不直接输出 protocol stdout；由 core CLI 完成输出层选择 | 不适用 | ownership 和 schema 验收 |

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
- structuredContent 使用 operation 对应的精简 readable schema，服务工具声明和客户端展示，不替代完整协议接口；存在兼容性 warning 时必须包含顶层 `warnings` 数组。
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
- adapter 选择候选 warning 在默认阅读文本、`readable-json` 和 MCP 中跟随最终阅读结果输出；在 `protocol-json` 中写 stderr，不能污染 stdout envelope。
- 直接 CLI argv 的兼容和 warning 归属见 [直接 CLI 兼容参数规则](#直接-cli-兼容参数规则)；通道承载必须与该规则一致。
- `config get` 的 key 不存在时必须返回 `INVALID_REQUEST`。
- 成功退出 `0`；输入错误 `2`；文档/ref/格式错误 `3`；协议或 adapter 进程错误 `4`；内部错误 `1`。

## Page 分页

- 分页操作的 page 是正整数；省略时固定为 `1`，且配置不能改变初始页。
- 结果固定返回下一页 page；null 表示没有更多信息。
- page 非 null 时，调用方保持 path、ref、query、limit_chars 和 options 不变，并把返回的 page 原样用于下一次请求。
- 非 null page 必须等于请求 page 加 1；请求超过末尾时返回空结果和 null。
- CLI、readable JSON 和 MCP 只返回 page 字段表达继续位置。
