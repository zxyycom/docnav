# CLI

本文是 `docnav` 命令、配置域、adapter 管理、adapter 直接 CLI、argv 兼容规则、退出码和分页入口的主规范。输出模式的序列化、warning 承载和 readable rendering 见 [输出模式](output.md)；MCP tool handoff 见 [MCP Handoff](mcp.md)。

## `docnav` 核心 CLI

`docnav` 提供所有接入方式共享的核心能力入口：

```text
docnav outline <path> [--adapter <adapter-id>] [--page 1] [--limit-chars 6000] [--output readable-view|readable-json|protocol-json]
docnav read <path> --ref <ref> [--adapter <adapter-id>] [--page 1] [--limit-chars 6000] [--output readable-view|readable-json|protocol-json]
docnav find <path> --query <text> [--adapter <adapter-id>] [--page 1] [--limit-chars 6000] [--output readable-view|readable-json|protocol-json]
docnav info <path> [--adapter <adapter-id>] [--output readable-view|readable-json|protocol-json]
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

Rust CLI argv 结构解析以 `clap` 或 `clap` builder API 为基础。`clap` 承载命令、子命令、固定参数、默认值、枚举值和 help；Docnav 在 command/operation 确定后只对当前 operation 实际使用的参数做类型、范围和枚举校验。成功解析的 document CLI argv 先进入 canonical document operation input 或等价 semantic request，再进入 adapter routing、invoke request 构造和 output dispatch。该内部语义输入不是 protocol envelope、schema 稳定类型或 MCP 传输模型。

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

`docnav` core 配置域拥有 `defaults.adapter`、`defaults.limit_chars` 和 `defaults.output`。adapter 配置域拥有格式解析参数、格式默认值和 adapter 直接 CLI 文案；MCP package 配置键由 MCP bridge owner 定义，core CLI 不拥有这些键。

配置只能控制所属 CLI 明确声明的行为默认值。配置不得改变 protocol-json 字段；readable-json 和 MCP structuredContent 的字段形状用于阅读输出和工具声明校验，不作为完整机器协议。阅读文案、MCP TextContent 包装文本或 tool 暴露策略如需配置，必须由对应 owner 文档定义键名、优先级和验证规则。

`docnav config set` 和 `unset` 默认写项目配置；传入 `--user` 时写用户配置。`config list` 不带 path 时列出 core 配置域的当前生效值；`config list --path <path> [--operation outline|read|find|info]` 解析文档上下文，展示该 path 触发的 adapter、core 参数来源和最终默认参数。

## Adapter 管理

`docnav adapter install <source>`、`update [adapter-id]` 和 `remove <adapter-id>` 是正式管理能力，不是占位命令。

首期 `install <source>` 只支持两类来源：

- 内置 adapter 下载简写：例如 `markdown`。`docnav` 必须解析为可执行 adapter 制品，执行 `manifest`，校验 manifest schema、必需字段和协议字段 shape，并记录 source key、解析后的制品信息、manifest 快照和可执行入口。
- 本地可执行文件：指向 adapter exe 的本地路径。`docnav` 必须解析为项目外部或项目内部的绝对可执行路径，执行 `manifest`，校验 manifest schema、必需字段和协议字段 shape，计算并记录可执行文件 SHA-256 fingerprint。普通文档操作不为 fingerprint 校验读取整个 adapter 可执行文件；install、update 和显式健康检查必须重新计算 fingerprint，fingerprint 不一致时不得静默继续使用旧安装记录。

- `list` 输出已安装 adapter、manifest 身份、支持格式、安装来源和可用状态。
- `install <source>` 校验失败不得注册；本地可执行文件缺失、不可执行或 fingerprint 无法计算时必须失败。
- `update [adapter-id]` 使用已记录来源获取或重新验证候选制品。内置下载来源重新走内置映射；本地可执行文件来源重新读取同一路径，重新计算 fingerprint，并在 manifest schema 和协议字段 shape 校验通过后更新记录。校验失败时保留旧记录并返回结构化错误。
- `remove <adapter-id>` 注销 adapter 并清理 `docnav` 管理的安装记录；仍被项目配置显式引用时必须失败或给出明确 guidance。

## Adapter 直接 CLI

```text
docnav-markdown outline <path> [--page 1] [--limit-chars 6000] [--max-heading-level 3] [--output readable-view|readable-json|protocol-json]
docnav-markdown read <path> --ref <ref> [--page 1] [--limit-chars 6000] [--output readable-view|readable-json|protocol-json]
docnav-markdown find <path> --query <text> [--page 1] [--limit-chars 6000] [--output readable-view|readable-json|protocol-json]
docnav-markdown info <path> [--output readable-view|readable-json|protocol-json]
docnav-markdown manifest [--output protocol-json]
docnav-markdown probe <path> [--output protocol-json]
docnav-markdown invoke
```

Adapter 直接 CLI 默认值由各 adapter 配置域拥有。`docnav-<adapter-id>` 在执行业务逻辑前解析显式参数、项目配置、用户配置和内置默认值，并在进入 `invoke` 前把最终有限参数显式写入请求。`docnav-markdown` 默认值见 [Markdown Adapter](adapters/markdown.md#默认值)。

分页操作省略 page 时固定读取第一页，并输出下一页 page 或 null。adapter 文档操作默认使用 `readable-view`；`protocol-json` 使用原始协议 envelope；`manifest` 和 `probe` 使用各自专属 schema。输出共享库的所有权见 [架构](architecture.md#共享库) 和 [输出模式](output.md#输出层边界)；CLI 本节只定义入口、参数和命令族边界。

## 直接 CLI 兼容参数规则

所有 Docnav 直接 CLI argv 使用同一兼容规则：`clap` 或 `clap` builder API 承载固定命令、已知参数、默认值、枚举值和 help；Docnav 或 SDK 在确定 command/operation 后只校验当前 operation 实际使用的参数。未知 flag、多余 positional、当前 operation 不使用的已知 flag 写 warning 后忽略；warning 使用稳定 envelope，并将相关原始 token 放入 `details.tokens`。`--unknown=value` 可作为一个未知 flag token 记录；`--unknown value` 的具体 token 分组和消费顺序不是稳定契约。当前 operation 不使用的已知有值 flag 只按 flag 形状消费并写 warning，不校验该 value 的业务合法性。已知必需参数缺失、当前 operation 实际使用的已知 flag 缺少值或值非法必须失败。

兼容规则只适用于直接 CLI argv，不适用于 adapter `invoke` stdin JSON。`invoke` 请求仍按 protocol request schema 严格校验，未知字段或参数类型错误不得被兼容忽略。

`docnav-cli-args` 拥有 loose token classification。core CLI 和 adapter SDK 仍分别拥有当前 operation 的 typed argument validation、默认值合并、业务 request 构造和最终 exit behavior；warning 的承载位置仍按本节和 [输出模式](output.md) 决定。

## 命令族矩阵

| 命令族 | Owner | 文档 semantic request | 宽松 argv | ignored argv 诊断通道 | protocol-shaped stdout 边界 | help 是否执行业务 | 验收边界 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Core document operations：`docnav outline/read/find/info` | `docnav` core CLI | 是 | 是；只宽松 unknown、extra 和 unused known | readable-view/readable-json 承载 warning；protocol-json 写 stderr | `--output protocol-json` stdout 只含 protocol response envelope | 否 | CLI 与输出层共同验收 |
| Core non-document commands：`config/init/doctor/version` | `docnav` core CLI | 否 | 类型化命令；可对无关 argv 给诊断 | 当前命令输出层或 stderr | 不产生 document protocol response | 否 | 代表性验收 |
| Core adapter management：`docnav adapter list/install/update/remove` | adapter management owner | 否 | 由 adapter management owner 定义 | 由 adapter management owner 定义 | 管理命令 schema/通道由 owner 定义 | 否 | 管理命令 owner 验收 |
| Adapter direct document operations：`docnav-markdown outline/read/find/info` | adapter SDK + format adapter | 是 | 是；只宽松 unknown、extra 和 unused known/native | readable-view/readable-json 承载 warning；protocol-json 写 stderr | `--output protocol-json` stdout 只含 protocol response envelope | 否 | adapter SDK 与格式 adapter 验收 |
| Adapter direct machine commands：`manifest/probe/invoke` | adapter SDK | `invoke` valid request 进入等价 semantic request；manifest/probe 否 | manifest/probe CLI argv 可宽松；invoke JSON 严格 | manifest/probe warning 写 stderr；invoke transport error 写 protocol failure | stdout 只含 manifest、probe 或 protocol response schema payload | 否 | adapter contract 验收 |
| Help commands：root help 和子命令 help | 各 CLI owner | 否 | 不适用 | stdout/stderr 只输出 help 文本 | 不输出 protocol/readable payload | 否 | CLI owner 验收 |
| MCP bridge 目标：`document_outline/read/find/info` | `implement-docnav-mcp-bridge` | 否，目标是映射为核心 `docnav` CLI | MCP arguments 不按 argv 宽松处理 | MCP structuredContent 复用 readable warning envelope | 不直接输出 protocol stdout；由 core CLI 完成输出层选择 | 不适用 | MCP handoff owner 验收 |

## 通道与退出码

- `readable-view` 和 `readable-json` 写 stdout。
- `protocol-json` 写 stdout，且只输出一个 JSON 值。
- 诊断写 stderr。
- adapter 选择候选 warning 在 `readable-view`、`readable-json` 和 MCP 中跟随最终阅读结果输出；在 `protocol-json` 中写 stderr，不能污染 stdout envelope。
- 直接 CLI argv 的兼容和 warning 归属见 [直接 CLI 兼容参数规则](#直接-cli-兼容参数规则)；通道承载必须与该规则一致。
- `config get` 的 key 不存在时必须返回 `INVALID_REQUEST`。
- 成功退出 `0`；输入错误 `2`；文档/ref/格式错误 `3`；协议或 adapter 进程错误 `4`；内部错误 `1`。

## Page 分页

- 分页操作的 page 是正整数；省略时固定为 `1`，且配置不能改变初始页。
- 结果固定返回下一页 page；null 表示没有更多信息。
- page 非 null 时，调用方保持 path、ref、query、limit_chars 和 options 不变，并把返回的 page 原样用于下一次请求。
- 非 null page 必须等于请求 page 加 1；请求超过末尾时返回空结果和 null。
- CLI、readable JSON 和 MCP 只返回 page 字段表达继续位置。
