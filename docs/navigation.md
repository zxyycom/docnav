# 文档导航

本文是 Docnav 的文档导航入口，负责说明如何按任务读取文档、各类文档的分层、关键规则 owner 和常用术语。项目概览和快速开始见 [README](../README.md)。

## 如何阅读这些文档

所有人先读 [README](../README.md) 了解项目目标和首期范围，再读 [架构](architecture.md)。之后按任务分流：

| 角色 / 任务 | 必读 | 需要时再读 |
| --- | --- | --- |
| 实现 `docnav` 核心 CLI | [架构](architecture.md)、[CLI 与 MCP 输出](cli.md) | [原始协议](protocol.md)、[测试策略](testing.md) |
| 实现 `docnav-mcp` | [CLI 与 MCP 输出](cli.md)、[原始协议](protocol.md) | [Schema](schemas/README.md)、[示例](examples/README.md) |
| 实现协议或 SDK | [原始协议](protocol.md) | [Schema](schemas/README.md)、[适配器契约](adapter-contract.md) |
| 实现格式 adapter | [适配器契约](adapter-contract.md)、[Ref](refs.md)、[原始协议](protocol.md) | [MarkdownNavigator 参考](references/markdown-navigator.md) |
| 写测试或验证脚本 | [测试策略](testing.md)、[Schema](schemas/README.md)、[示例](examples/README.md) | 对应实现面的主规范 |
| 审计历史或变更依据 | `../openspec/changes/` | 按 change 目录读取对应 proposal、design、specs、tasks |

OpenSpec 是变更设计、验收和审计历史，不作为日常实现的主入口。日常开发优先从 README、本导航页、架构和对应任务主规范进入。

## 常用验证入口

交付前的综合验证优先运行：

```bash
pnpm run verify:docnav-workspace
```

该命令汇总常用门禁；详细规则和局部验证取舍见 [测试策略](testing.md)。

## 文档分层

| 类型 | 文档 | 用途 |
| --- | --- | --- |
| 项目首页 | [README](../README.md) | 项目目标、v0 范围、快速开始、验证入口和文档入口 |
| 文档导航 | 本文档 | 角色化阅读路径、文档分层、规则 owner 和术语 |
| 主规范 | [架构](architecture.md)、[CLI 与 MCP 输出](cli.md)、[原始协议](protocol.md)、[适配器契约](adapter-contract.md)、[Ref](refs.md)、[测试策略](testing.md) | 定义稳定规则和实现职责 |
| 校验材料 | [Schema](schemas/README.md)、[示例](examples/README.md) | 校验主规范中的字段形状、示例链路和输出映射 |
| 工程规范 | [编码规范](CODING_STYLE.md) | 约束实现风格、边界错误、模块组织和提交前检查 |
| 参考材料 | [MarkdownNavigator 参考](references/markdown-navigator.md) | Markdown 行为来源和迁移决策 |
| 变更历史 | `../openspec/changes/` | change 设计与验收历史 |

Schema 和示例是验证材料，不是新的规范来源；当示例与主规范不一致时，以主规范为准，并修正示例或 schema。用于生成代码、schema 或校验常量的机器规则文件只服务对应主规范的落地，不独立拥有产品语义。

## 规则所有权

关键规则只由一个主文档拥有，其它文档只摘要或引用，避免重复定义造成漂移。

| 规则面 | Owner 文档 |
| --- | --- |
| 制品职责、调用链、adapter 选择、配置所有权、进程边界 | [架构](architecture.md) |
| `docnav` 命令、adapter 管理命令、输出模式、MCP 映射、退出码 | [CLI 与 MCP 输出](cli.md) |
| invoke envelope、operation、紧凑结果、page、稳定错误 | [原始协议](protocol.md) |
| adapter 命令、manifest、probe、格式默认值、invoke 行为 | [适配器契约](adapter-contract.md) |
| ref 的生成、定位、唯一性和原样传递 | [Ref](refs.md) |
| 自动化测试层级、验收矩阵、一致性审计 | [测试策略](testing.md) |
| JSON 字段形状和示例语义校验 | [Schema](schemas/README.md)、[示例](examples/README.md) |

## 术语

| 术语 | 定义 |
| --- | --- |
| document | 由项目相对路径标识的输入文档。 |
| outline | 格式适配器生成的扁平导航条目列表。 |
| entry | outline 中的一条紧凑阅读记录，至少包含 `ref` 和 `display`。 |
| path | 定位文档，并作为 `docnav` 选择 adapter 的依据。 |
| ref | 由适配器生成和解析，只定位当前文档内部区域。 |
| limit_chars | 每页语义结果的字符预算。 |
| content_type | adapter 返回的内容类型，例如 `text/markdown`；readable read 和 MCP read structuredContent 保留该字段。 |
| page | 请求时表示要读取的页码；响应时表示下一页页码，null 表示没有更多信息。 |
| docnav | 核心 CLI，负责识别、路由、分发、管理、配置和项目初始化。 |
| read | 使用 ref 唯一读取文档区域。 |
| adapter | 负责一种或一组格式解析与导航的独立可执行制品。 |
| invoke | 适配器单请求原始协议入口。 |
| manifest | 适配器身份、协议范围、格式、能力和推荐参数声明。 |
| probe | 适配器对文档格式支持度及判断依据的结果。 |
| capability | 适配器声明支持的 `outline`、`read`、`find` 或 `info`。 |
| readable output | 面向 AI 或人类的信息密集输出，以可读性为主，不承诺作为机器兼容解析接口。 |
| protocol output | 面向 `docnav`、脚本、调试和兼容校验的稳定 envelope，不以可读性为目标。 |
