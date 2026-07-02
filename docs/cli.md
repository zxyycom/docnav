# CLI

本文是 `docnav` 命令、`config` 命令入口、内置 adapter inspection、退出码和分页入口的主规范。输出模式的序列化、诊断投影承载和 readable rendering 见 [输出模式](output.md)；直接 CLI argv 映射、配置映射和标准参数机制见 [标准参数](standard-parameters.md)。

## `docnav` 核心 CLI

`docnav` 提供所有调用入口共享的核心能力入口：

```text
docnav outline <path> [--adapter <adapter-id>] [--pagination enabled|disabled] [--page 1] [--limit 6000] [--output readable-view|readable-json|protocol-json]
docnav read <path> --ref <ref> [--adapter <adapter-id>] [--pagination enabled|disabled] [--page 1] [--limit 6000] [--output readable-view|readable-json|protocol-json]
docnav find <path> --query <text> [--adapter <adapter-id>] [--pagination enabled|disabled] [--page 1] [--limit 6000] [--output readable-view|readable-json|protocol-json]
docnav info <path> [--adapter <adapter-id>] [--output readable-view|readable-json|protocol-json]
docnav init
docnav doctor
docnav config get|set|unset|list [--user] [--path <path>] [--operation outline|read|find|info]
docnav adapter list
docnav version
```

`docnav` 在调用 adapter layer 前完成：

1. 项目根解析。
2. path 规范化和可访问性检查。
3. adapter 选择：显式 `--adapter` 或配置提供的 adapter id 按 declared intent 在 static registry 中查找并 probe；未声明 adapter 时才进入 automatic discovery，并按 registry 顺序 probe 返回第一个成功项。显式 adapter id 不存在时返回 adapter selection diagnostic，优先于 native option validation。完整规则见 [适配器契约](adapter-contract.md#adapter-选择)。
4. 使用 core 标准参数 registration 解析显式 argv、项目配置、用户配置和 core 内置默认值。
5. pagination enabled 状态、page、limit、output 和其它 core-owned 标准参数解析。
6. 输出模式和错误映射选择。

Rust CLI argv 结构解析以 `clap` 或 `clap` builder API 为基础。`clap` 承载命令、子命令、固定参数、默认值、枚举值和 help；core 标准参数 registration 承接 flag 映射、校验、help/default 文案和 operation argument binding。Docnav 在 command/operation 确定后只校验当前 operation 实际使用的参数。成功解析的 document CLI argv 进入标准参数机制，与配置和默认值一起产出 core 参数结果；随后进入 adapter routing、protocol request 构造和 output dispatch。该内部语义输入不是 protocol envelope 或 schema 稳定类型。

未知 argv、多余 positional 和当前 operation 不使用的已知参数表达 invalid caller intent，必须在 document operation execution 和 adapter routing 前返回输入错误并写入错误通道。当前 operation 实际使用的 core 参数同样严格校验：缺少必需 path/ref/query、非法 page、非法 limit 或非法 output 必须返回输入错误。Native option 输入通过源码级 registry 进入 generic merge；core 在 adapter selection 后按 selected adapter descriptor 判断支持性，不支持的 option 返回 native option diagnostic；类型、范围和格式语义由 adapter 在消费时返回 type mismatch 或 range invalid 的结构化诊断。当前 operation 不使用的 known 参数只需要按 token/operation membership 产生 applicability diagnostic，不触发其它 operation 的 eager typed validation。

### 项目根与路径

`docnav` 按以下顺序确定项目根：

1. 从启动 cwd 向上查找最近的 `.docnav/`。
2. 未找到时使用启动 cwd。

Document operation、`init`、`doctor` 和 `config` 命令使用该项目根解析项目配置和项目上下文。`docnav` 接受项目根内外的可访问文件路径。相对 path 基于启动 cwd 解析；`document.path` 必须使用 `/`，项目根内路径可以传项目相对路径，项目根外路径传规范化绝对路径。路径不存在、不可读或无法规范化时返回文档路径错误，不能调用 adapter layer。

## 配置命令边界

`docnav config get|set|unset|list` 是 core CLI 命令族。配置字段映射、supported key、配置验证、来源合并和 `config list --path <path> [--operation outline|read|find|info]` 的来源展示由 [标准参数](standard-parameters.md#输入与配置映射) 定义。

CLI 本节只定义命令入口和退出边界：

- `docnav config set` 和 `unset` 默认写项目配置；传入 `--user` 时写用户配置。
- `config get` 的 key 不存在时返回 `INVALID_REQUEST`。
- `config list --path <path> [--operation outline|read|find|info]` 可以解析文档上下文；触发的 adapter、标准参数来源和最终值由标准参数机制提供。
- `config` 命令不产生 document protocol response。

## 内置 adapter 检查

`docnav adapter list` 是 core release 内置 adapter inspection。它的数据源固定为随当前 release 编译的 static registry，只展示 adapter layer metadata，例如 adapter id、名称、版本、支持格式、扩展名、content type 和 operation metadata。

默认 adapter 命令面只包含 `docnav adapter list`。

`docnav doctor` 检查项目/用户配置、static registry 和 core release 内置 adapter layer 可用性。doctor 的 adapter layer check 可以调用 registry 中 adapter 的 metadata 支持逻辑，验证静态 descriptor 与 linked handler 是否一致；修复建议应落在当前配置、static registry 或 linked adapter layer 边界内。

## adapter 执行入口

默认 CLI surface 的 adapter 执行入口是 core-linked library handle。格式 adapter 作为 core release 内置 workspace crate 暴露 `docnav-adapter-contracts::Adapter`；core CLI 通过 `docnav-navigation` 构造内部 protocol request 并调用 `outline/read/find/info` operation handlers。

格式 adapter 的默认值来自对应标准参数 definition 和源码级 static option registry。`docnav` document operation 在构造 request 前完成 argv、配置、native option source 和内置默认值解析；request construction 只序列化 protocol 需要的字段，以及当前入口明确保留的 adapter-owned options。完整映射规则见 [标准参数](standard-parameters.md)，`docnav-markdown` 默认值见 [Markdown Adapter](adapters/markdown.md#默认值)。

分页操作省略 page 时固定读取第一页，并输出下一页 page 或 null。文档操作默认使用 `readable-view`；`protocol-json` 使用原始协议 envelope。输出共享库的所有权见 [架构](architecture.md#共享库) 和 [输出模式](output.md#输出层边界)；CLI 本节只定义入口、参数和命令族边界。

## 直接 CLI argv 边界

直接 CLI argv 的 strict classification、known value flag metadata、operation membership、unmapped token diagnostics 和 typed validation 由 [标准参数](standard-parameters.md#输入与配置映射) 定义。CLI 本节只约束入口边界：core CLI 保留 `clap` 作为 parser/help owner，提供 command context 与 registration metadata，消费标准参数结果，并负责 request construction、operation build、primary `DiagnosticRecord` 投影与最终 exit behavior。

## 命令族矩阵

| 命令族 | Owner | 文档 semantic request | strict argv | invalid input 投影通道 | protocol-shaped stdout 边界 | help 是否执行业务 | 验收边界 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Core document operations：`docnav outline/read/find/info` | `docnav` core CLI | 是 | 是；unknown、extra 和 unused known 在执行前失败 | readable-view/readable-json/protocol-json 按输出模式投影 primary failure | `--output protocol-json` stdout 只含 protocol response envelope | 否 | CLI 与输出层共同验收 |
| Core non-document commands：`config/init/doctor/version` | `docnav` core CLI | 否 | 类型化命令；无关 argv 按命令 owner 失败 | 成功输出走命令自有 PlainText/JSON；致命诊断走统一错误投影 | 成功不产生 document protocol result；错误按 output context 投影 | 否 | 代表性验收 |
| Core adapter inspection：`docnav adapter list` | `docnav` static registry owner | 否 | 类型化命令；无关 argv 按命令 owner 失败 | 成功输出走命令自有 JSON；致命诊断走统一错误投影 | 成功不产生 document protocol result；错误按 output context 投影 | 否 | static registry inspection 验收 |
| Help commands：root help 和子命令 help | 各 CLI owner | 否 | 不适用 | stdout/stderr 只输出 help 文本 | 不输出 protocol/readable payload | 否 | CLI owner 验收 |

## 通道与退出码

- `readable-view` 和 `readable-json` 写 stdout。
- `protocol-json` 写 stdout，且只输出一个 JSON 值。
- 诊断记录可投影到 stderr。
- adapter automatic discovery 的候选 probe 失败在 discovery 过程中保持为 internal state；若后续候选成功，成功输出只包含最终业务 payload；若全部候选失败，候选摘要作为 primary failure details 投影。
- 直接 CLI argv 的 strict 分类和输入诊断交接数据见 [标准参数](standard-parameters.md#错误出口)；通道承载必须与该规则一致。
- `config get` 的 key 不存在时必须返回 `INVALID_REQUEST`。
- 成功退出 `0`；输入错误 `2`；文档/ref/格式错误 `3`；协议或 adapter layer 错误 `4`；内部错误 `1`。退出码由 CLI owner 按错误通道记录的 code category/effect 投影映射。

## 分页入口

- 分页操作的 page 是正整数；省略时固定为 `1`，且配置不能改变初始页。
- 结果固定返回下一页 page；null 表示没有更多信息。
- page 非 null 时，调用方保持 path、ref、query、limit 和 options 不变，并把返回的 page 原样用于下一次请求。
- 非 null page 必须等于请求 page 加 1；请求超过末尾时返回空结果和 null。
- CLI 和 readable JSON 只返回 page 字段表达继续位置。
