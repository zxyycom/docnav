# 文档导航

改动前用本文定位任务主规范、规则 owner 和交付验证入口；规则细节进入对应 owner 文档。

## 如何阅读这些文档

按任务进入对应主规范；跨模块、边界或规则归属不清时补读 [架构](architecture.md) 和“规则所有权”。

| 角色 / 任务 | 必读 | 需要时再读 |
| --- | --- | --- |
| 实现 `docnav` 核心 CLI | [架构](architecture.md)、[CLI](cli.md)、[输出模式](output.md) | [原始协议](protocol.md)、[测试策略](testing.md) |
| 实现 `docnav-mcp` | [MCP Handoff](mcp.md)、[输出模式](output.md)、[原始协议](protocol.md) | [JSON Schema 索引](schemas/json-schema.md)、[示例](examples/README.md) |
| 实现协议或 SDK | [原始协议](protocol.md) | [JSON Schema 索引](schemas/json-schema.md)、[适配器契约](adapter-contract.md) |
| 实现 Markdown adapter | [适配器契约](adapter-contract.md)、[Ref](refs.md)、[原始协议](protocol.md)、[Markdown Adapter](adapters/markdown.md) | [MarkdownNavigator 参考](references/markdown-navigator.md) |
| 写测试或验证脚本 | [测试策略](testing.md)、[工程工具链](tooling.md)、[JSON Schema 索引](schemas/json-schema.md)、[示例](examples/README.md) | [测试用例编号账本](testing/cases.md)、[覆盖矩阵](testing/coverage.md)、对应实现面的主规范 |
| 审计历史或变更依据 | `../openspec/changes/` | 按 change 目录读取对应 proposal、design、specs、tasks |

`../openspec/changes/` 用于变更设计、验收和审计历史；日常实现从对应任务主规范进入。

## 交付验证

交付前默认运行：

```bash
pnpm run verify:docnav-workspace
```

局部验证取舍见 [测试策略](testing.md)，脚本工具链和本地工具运行方式见 [工程工具链](tooling.md)。

## 文档分层

| 类型 | 文档 | 使用时机 |
| --- | --- | --- |
| 项目首页 | [README](../README.md) | 确认项目目标、v0 范围或运行入口 |
| 文档导航 | 本文档 | 定位任务主规范、规则 owner 和交付验证入口 |
| 主规范 | [架构](architecture.md)、[CLI](cli.md)、[输出模式](output.md)、[MCP Handoff](mcp.md)、[原始协议](protocol.md)、[适配器契约](adapter-contract.md)、[Ref](refs.md)、[测试策略](testing.md) | 修改稳定规则或实现职责 |
| 测试资料 | [测试用例编号账本](testing/cases.md)、[覆盖矩阵](testing/coverage.md)、[发布包验证](testing/release.md) | 调整测试编号、覆盖目标或 release 验证 |
| Adapter 专页 | [Markdown Adapter](adapters/markdown.md) | 修改 Markdown adapter 私有行为 |
| 校验材料 | [JSON Schema 索引](schemas/json-schema.md)、[示例](examples/README.md) | 修改字段形状、示例链路或输出映射 |
| 工程规范 | [编码规范](CODING_STYLE.md)、[工程工具链](tooling.md) | 修改代码、脚本或验证工具链 |
| 参考材料 | [MarkdownNavigator 参考](references/markdown-navigator.md) | 追溯 Markdown 行为来源或迁移依据 |
| 变更历史 | `../openspec/changes/` | 审计 change 设计、验收或历史决策 |

Schema、示例和机器规则文件是验证材料；与主规范不一致时修正验证材料，不在这些文件重新定义产品语义。

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
| 测试用例编号、证明目标和源码 `@case` 标记映射 | [测试用例编号账本](testing/cases.md) |
| 脚本语言、包管理、Node.js/TypeScript 脚本运行方式和脚本类型检查门禁 | [工程工具链](tooling.md) |
| JSON 字段形状和示例语义校验 | [JSON Schema 索引](schemas/json-schema.md)、[示例](examples/README.md) |

## 术语

本节保留影响任务路由和规则 owner 判断的跨文档词；完整规则进入上方 owner 文档。

| 术语 | 定义 |
| --- | --- |
| owner 文档 | 某类规则的完整解释和维护位置；其它文档只保留摘要或引用。 |
| docnav | 核心 CLI，负责格式识别、adapter 路由、配置、管理和输出分发。 |
| adapter | 独立格式处理制品，拥有格式解析、导航策略、ref 和分页语义。 |
| document | Docnav 操作的输入文件，由 path 定位并用于 adapter 选择。 |
| `outline -> ref -> read` | 标准导航流程：先取结构条目，再把 adapter 生成的 ref 原样传回读取。 |
| ref | adapter 生成和解析的非空 opaque string；共享层只原样传递。 |
| readable output | 面向人类和 AI 的阅读输出，包括 `readable-view` 和 `readable-json`；规则见 [输出模式](output.md)。 |
| protocol output | 面向脚本、调试和兼容校验的稳定 envelope；协议语义见 [原始协议](protocol.md)，CLI 模式见 [输出模式](output.md)。 |
