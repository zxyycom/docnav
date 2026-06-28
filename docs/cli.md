# CLI

本文是 `docnav` 命令、`config` 命令入口、adapter 管理、adapter 直接 CLI、退出码和分页入口的主规范。输出模式的序列化、诊断投影承载和 readable rendering 见 [输出模式](output.md)；直接 CLI argv 映射、配置映射和标准参数机制见 [标准参数](standard-parameters.md)。

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
4. 使用 core 标准参数 registration 解析显式 argv、项目配置、用户配置和 core 内置默认值。
5. page、limit_chars、output 和其它 core-owned 标准参数解析。
6. 输出模式和错误映射选择。

Rust CLI argv 结构解析以 `clap` 或 `clap` builder API 为基础。`clap` 承载命令、子命令、固定参数、默认值、枚举值和 help；core 标准参数 registration 承接 flag 映射、校验、help/default 文案和 operation argument binding。Docnav 在 command/operation 确定后只校验当前 operation 实际使用的参数。成功解析的 document CLI argv 进入标准参数机制，与配置和默认值一起产出 core 参数结果；随后进入 adapter routing、invoke request 构造和 output dispatch。该内部语义输入不是 protocol envelope 或 schema 稳定类型。

未知 argv、多余 positional 和当前 operation 不使用的已知参数不阻断有效文档操作；它们只生成可恢复诊断交接数据。当前 operation 实际使用的参数仍严格校验：缺少必需 path/ref/query、非法 page、非法 limit_chars、非法 output 或非法 native option 必须返回输入错误并写入错误通道。当前 operation 不使用的 known 参数即使值不符合其它 operation 的类型、范围或枚举规则，也不得触发 eager typed failure。

## 配置命令边界

`docnav config get|set|unset|list` 是 core CLI 命令族。配置字段映射、supported key、配置验证、来源合并和 `config list --path <path> [--operation outline|read|find|info]` 的来源展示由 [标准参数](standard-parameters.md#输入与配置映射) 定义。

CLI 本节只定义命令入口和退出边界：

- `docnav config set` 和 `unset` 默认写项目配置；传入 `--user` 时写用户配置。
- `config get` 的 key 不存在时返回 `INVALID_REQUEST`。
- `config list --path <path> [--operation outline|read|find|info]` 可以解析文档上下文；触发的 adapter、标准参数来源和最终值由标准参数机制提供。
- `config` 命令不产生 document protocol response。

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
docnav-markdown outline <path> [--page 1] [--limit-chars 6000] [--max-heading-level 3] [--output readable-view|readable-json|protocol-json] [--project-config-path <path>] [--user-config-path <path>]
docnav-markdown read <path> --ref <ref> [--page 1] [--limit-chars 6000] [--output readable-view|readable-json|protocol-json] [--project-config-path <path>] [--user-config-path <path>]
docnav-markdown find <path> --query <text> [--page 1] [--limit-chars 6000] [--output readable-view|readable-json|protocol-json] [--project-config-path <path>] [--user-config-path <path>]
docnav-markdown info <path> [--output readable-view|readable-json|protocol-json] [--project-config-path <path>] [--user-config-path <path>]
docnav-markdown manifest [--output protocol-json]
docnav-markdown probe <path> [--output protocol-json]
docnav-markdown invoke
```

Adapter 直接 CLI 默认值来自对应标准参数 definition 和 registration。`docnav-<adapter-id>` document operation 在构造 request 前，按标准参数机制解析显式 argv、adapter 配置和内置默认值；request construction 只序列化 protocol 需要的显式字段，以及当前入口明确保留的透传字段。Adapter `invoke` 作为独立 protocol 入口重新处理 request。`docnav-markdown` 默认值见 [Markdown Adapter](adapters/markdown.md#默认值)。

Adapter direct CLI 配置路径发现、字段映射、来源合并和配置源失败规则由 [标准参数](standard-parameters.md#输入与配置映射) 定义。被跳过的配置源先形成可恢复诊断，再由 [输出模式](output.md#readable-json) 投影为 readable warning 或 stderr 诊断。

`manifest`、`probe` 和 help 不读取 adapter direct CLI document operation 配置。Document operation help 必须展示 `--project-config-path <path>` 和 `--user-config-path <path>`；help 只输出参数说明，不执行配置读取或文档导航。`invoke` 只消费 stdin 中的 protocol request，不执行 adapter direct CLI argv parsing 或 help；request `arguments` 是 `invoke` 的 direct input，后续映射、配置定位和 request construction 交接见 [标准参数](standard-parameters.md#metadata-与交接边界)。

分页操作省略 page 时固定读取第一页，并输出下一页 page 或 null。adapter 文档操作默认使用 `readable-view`；`protocol-json` 使用原始协议 envelope；`manifest` 和 `probe` 使用各自专属 schema。输出共享库的所有权见 [架构](architecture.md#共享库) 和 [输出模式](output.md#输出层边界)；CLI 本节只定义入口、参数和命令族边界。

## 直接 CLI argv 边界

直接 CLI argv 的 loose classification、known value flag metadata、operation membership、未使用条目诊断交接数据和 typed validation 由 [标准参数](standard-parameters.md#输入与配置映射) 定义。CLI 本节只约束入口边界：core CLI 和 adapter SDK 提供 command context 与 registration metadata，消费标准参数结果，并负责各自的 request construction、operation build、诊断投影与刷新和最终 exit behavior。

## 命令族矩阵

| 命令族 | Owner | 文档 semantic request | 宽松 argv | ignored argv 诊断投影通道 | protocol-shaped stdout 边界 | help 是否执行业务 | 验收边界 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Core document operations：`docnav outline/read/find/info` | `docnav` core CLI | 是 | 是；只宽松 unknown、extra 和 unused known | readable-view/readable-json 承载 warning 投影；protocol-json 写 stderr | `--output protocol-json` stdout 只含 protocol response envelope | 否 | CLI 与输出层共同验收 |
| Core non-document commands：`config/init/doctor/version` | `docnav` core CLI | 否 | 类型化命令；可对无关 argv 写入诊断记录 | 成功输出走命令自有 PlainText/JSON；致命诊断走统一错误投影 | 成功不产生 document protocol result；错误按 output context 投影 | 否 | 代表性验收 |
| Core adapter management：`docnav adapter list/install/update/remove` | adapter management owner | 否 | 由 adapter management owner 定义 | 由 adapter management owner 定义 | 管理命令 schema/通道由 owner 定义 | 否 | 管理命令 owner 验收 |
| Adapter direct document operations：`docnav-markdown outline/read/find/info` | adapter SDK + format adapter | 是 | 是；只宽松 unknown、extra 和 unused known/native | readable-view/readable-json 承载 warning 投影；protocol-json 写 stderr | `--output protocol-json` stdout 只含 protocol response envelope | 否 | adapter SDK 与格式 adapter 验收 |
| Adapter direct machine commands：`manifest/probe/invoke` | adapter SDK | `invoke` valid request 进入等价 semantic request；manifest/probe 否 | manifest/probe CLI argv 可宽松；invoke JSON 严格 | manifest/probe 诊断写 stderr；invoke transport error 写 protocol failure | stdout 只含 manifest、probe 或 protocol response schema payload | 否 | adapter contract 验收 |
| Help commands：root help 和子命令 help | 各 CLI owner | 否 | 不适用 | stdout/stderr 只输出 help 文本 | 不输出 protocol/readable payload | 否 | CLI owner 验收 |

## 通道与退出码

- `readable-view` 和 `readable-json` 写 stdout。
- `protocol-json` 写 stdout，且只输出一个 JSON 值。
- 诊断记录可投影到 stderr。
- adapter 选择候选记录在 `readable-view` 和 `readable-json` 中跟随最终阅读结果投影为 warning；在 `protocol-json` 中写 stderr，不能污染 stdout envelope。
- 直接 CLI argv 的兼容分类和诊断交接数据见 [标准参数](standard-parameters.md#错误出口)；通道承载必须与该规则一致。
- `config get` 的 key 不存在时必须返回 `INVALID_REQUEST`。
- 成功退出 `0`；输入错误 `2`；文档/ref/格式错误 `3`；协议或 adapter 进程错误 `4`；内部错误 `1`。退出码由 CLI owner 按错误通道记录的 code category/effect 投影映射。

## Page 分页

- 分页操作的 page 是正整数；省略时固定为 `1`，且配置不能改变初始页。
- 结果固定返回下一页 page；null 表示没有更多信息。
- page 非 null 时，调用方保持 path、ref、query、limit_chars 和 options 不变，并把返回的 page 原样用于下一次请求。
- 非 null page 必须等于请求 page 加 1；请求超过末尾时返回空结果和 null。
- CLI 和 readable JSON 只返回 page 字段表达继续位置。
