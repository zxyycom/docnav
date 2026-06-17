# 文档导航

本文是 Docnav 的文档导航入口，负责说明如何按任务读取文档、各类文档的分层、关键规则 owner 和常用术语。项目概览和快速开始见 [README](../README.md)。

## 如何阅读这些文档

所有人先读 [README](../README.md) 了解项目目标和首期范围，再读 [架构](architecture.md)。之后按任务分流：

| 角色 / 任务 | 必读 | 需要时再读 |
| --- | --- | --- |
| 实现 `docnav` 核心 CLI | [架构](architecture.md)、[CLI](cli.md)、[输出模式](output.md) | [原始协议](protocol.md)、[测试策略](testing.md) |
| 实现 `docnav-mcp` | [MCP Handoff](mcp.md)、[输出模式](output.md)、[原始协议](protocol.md) | [JSON Schema 索引](schemas/json-schema.md)、[示例](examples/README.md) |
| 实现协议或 SDK | [原始协议](protocol.md) | [JSON Schema 索引](schemas/json-schema.md)、[适配器契约](adapter-contract.md) |
| 实现 Markdown adapter | [适配器契约](adapter-contract.md)、[Ref](refs.md)、[原始协议](protocol.md)、[Markdown Adapter](adapters/markdown.md) | [MarkdownNavigator 参考](references/markdown-navigator.md) |
| 写测试或验证脚本 | [测试策略](testing.md)、[工程工具链](tooling.md)、[JSON Schema 索引](schemas/json-schema.md)、[示例](examples/README.md) | [覆盖矩阵](testing/coverage.md)、[Smoke Case 清单](testing/smoke-cases.md)、对应实现面的主规范 |
| 审计历史或变更依据 | `../openspec/changes/` | 按 change 目录读取对应 proposal、design、specs、tasks |

OpenSpec 是变更设计、验收和审计历史，不作为日常实现的主入口。日常开发优先从 README、本导航页、架构和对应任务主规范进入。

## 常用验证入口

交付前的综合验证优先运行：

```bash
pnpm run verify:docnav-workspace
```

该命令汇总常用门禁；profile 规则和局部验证取舍见 [测试策略](testing.md)，脚本工具链和本地工具运行方式见 [工程工具链](tooling.md)。

## 文档分层

| 类型 | 文档 | 用途 |
| --- | --- | --- |
| 项目首页 | [README](../README.md) | 项目目标、v0 范围、快速开始、验证入口和文档入口 |
| 文档导航 | 本文档 | 角色化阅读路径、文档分层、规则 owner 和术语 |
| 主规范 | [架构](architecture.md)、[CLI](cli.md)、[输出模式](output.md)、[MCP Handoff](mcp.md)、[原始协议](protocol.md)、[适配器契约](adapter-contract.md)、[Ref](refs.md)、[测试策略](testing.md) | 定义稳定规则和实现职责 |
| 测试资料 | [覆盖矩阵](testing/coverage.md)、[Smoke Case 清单](testing/smoke-cases.md)、[发布包验证](testing/release.md) | 维护测试覆盖目标、smoke case inventory 和 release package 验证边界 |
| Adapter 专页 | [Markdown Adapter](adapters/markdown.md) | Markdown adapter 私有导航行为、ref grammar、错误分类和验证入口 |
| 校验材料 | [JSON Schema 索引](schemas/json-schema.md)、[示例](examples/README.md) | 校验主规范中的字段形状、示例链路和输出映射 |
| 工程规范 | [编码规范](CODING_STYLE.md)、[工程工具链](tooling.md) | 约束实现风格、边界错误、模块组织、提交前检查、脚本工具链和本地验证工具运行方式 |
| 参考材料 | [MarkdownNavigator 参考](references/markdown-navigator.md) | Markdown 行为来源和迁移决策 |
| 变更历史 | `../openspec/changes/` | change 设计与验收历史 |

Schema 和示例是验证材料，不是新的规范来源；当示例与主规范不一致时，以主规范为准，并修正示例或 schema。用于生成代码、schema 或校验常量的机器规则文件只服务对应主规范的落地，不独立拥有产品语义。

## 规则所有权

关键规则只由一个主文档拥有，其它文档只摘要或引用，避免重复定义造成漂移。

| 规则面 | Owner 文档 |
| --- | --- |
| 制品职责、调用链、adapter 选择、配置所有权、进程边界 | [架构](architecture.md) |
| `docnav` 命令、adapter 管理命令、直接 CLI argv 兼容规则、退出码 | [CLI](cli.md) |
| 输出模式、readable-view framing、readable-json warning、阅读文案配置、输出通道 | [输出模式](output.md) |
| MCP target tools、tool 参数映射、TextContent 和 structuredContent 交接边界 | [MCP Handoff](mcp.md) |
| invoke envelope、operation、紧凑结果、page、稳定错误 | [原始协议](protocol.md) |
| adapter 命令、manifest、probe、格式默认值、invoke 行为 | [适配器契约](adapter-contract.md) |
| ref 的共享调用流程、非空 opaque string、原样传递和 adapter 所有权 | [Ref](refs.md) |
| Markdown ref grammar、结构快照语义、错误分类和显示职责 | [Markdown Adapter](adapters/markdown.md) |
| 自动化测试层级、覆盖目标、一致性审计和 release 验证边界 | [测试策略](testing.md)、[覆盖矩阵](testing/coverage.md)、[发布包验证](testing/release.md) |
| 脚本语言、包管理、Node.js/TypeScript 脚本运行方式和脚本类型检查门禁 | [工程工具链](tooling.md) |
| JSON 字段形状和示例语义校验 | [JSON Schema 索引](schemas/json-schema.md)、[示例](examples/README.md) |

## 术语

| 术语 | 定义 |
| --- | --- |
| document | 由调用方提供的规范 path 标识的输入文档，可以位于项目根内或项目根外。 |
| outline | 格式适配器生成的扁平导航条目列表。 |
| entry | outline 中的一条紧凑阅读记录，至少包含 `ref` 和 `display`。 |
| path | 定位文档，并作为 `docnav` 选择 adapter 的依据。 |
| ref | 由 adapter 生成和解析的非空 opaque string，共享层只原样传递。具体 grammar、定位语义和保证范围由 adapter 专属契约定义。 |
| limit_chars | 每页语义结果的字符预算。 |
| content_type | adapter 返回的内容类型，例如 `text/markdown`；readable read 和 MCP read structuredContent 保留该字段。 |
| page | 请求时表示要读取的页码；响应时表示下一页页码，null 表示没有更多信息。 |
| docnav | 核心 CLI，负责识别、路由、分发、管理、配置和项目初始化。 |
| read | 使用 adapter 生成的 ref 读取文档区域；共享层原样传递 ref，adapter 按其私有契约解释或拒绝。 |
| adapter | 负责一种或一组格式解析与导航的独立可执行制品。 |
| invoke | 适配器单请求原始协议入口。 |
| manifest | 适配器身份、支持格式、扩展名、content type 和 capabilities 声明。 |
| probe | 适配器对文档格式支持度及判断依据的结果。 |
| capability | 适配器声明支持的 `outline`、`read`、`find` 或 `info`。 |
| readable-view | 文档操作的默认阅读输出；完整格式、block 字段、byte framing 和 warning/error 规则由 [输出模式](output.md#readable-view) 定义。 |
| readable-json | 结构化阅读 JSON 输出，保持 documented shape，用于 AI、工具和轻量自动化解析阅读结果；不包含 protocol envelope。与 `readable-view` 从同一 typed readable payload 派生。 |
| renderer config | 仓库内提交的 readable-view block 字段声明；用户配置不得修改；完整规则见 [输出模式](output.md#readable-view)。 |
| conformance vector | 跨语言可消费的 JSON fixture，描述输入 payload、view kind、config override 和顺序无关断言（block pointer、byte length、block payload 还原、header 字段语义等）。 |
| readable output | 面向 AI 或人类的信息密集输出（`readable-view` 或 `readable-json`），以可读性为主，不承诺作为机器兼容解析接口。 |
| protocol-json | 文档操作的完整协议输出模式，stdout 使用 protocol response envelope，面向 `docnav`、脚本、调试和兼容校验，不以可读性为目标。 |
| protocol output | 面向 `docnav`、脚本、调试和兼容校验的稳定 envelope，不以可读性为目标；文档 CLI 中对应 `protocol-json`。 |
