# 文档导航

改动前用本文定位任务主规范、规则 owner 和交付验证入口；规则细节进入对应 owner 文档。

## 如何阅读这些文档

按任务进入对应主规范；跨模块、边界、状态或规则归属不清时补读 [架构](architecture.md)、“规范状态与实现状态”和“规则所有权”。

| 角色 / 任务 | 必读 | 需要时再读 |
| --- | --- | --- |
| 实现 `docnav` 核心 CLI | [架构](architecture.md)、[CLI](cli.md)、[标准参数](standard-parameters.md)、[输出模式](output.md) | [原始协议](protocol.md)、[测试策略](testing.md) |
| 实现协议或 SDK | [原始协议](protocol.md) | [JSON Schema 索引](schemas/json-schema.md)、[适配器契约](adapter-contract.md) |
| 实现 Markdown adapter | [适配器契约](adapter-contract.md)、[Ref](ref-contract.md)、[原始协议](protocol.md)、[Markdown Adapter](adapters/markdown.md) | 对应实现面的主规范 |
| 写测试或验证脚本 | [测试策略](testing.md)、[测试用例维护](testing/case-maintenance.md)、[工程工具链](tooling.md)、[JSON Schema 索引](schemas/json-schema.md)、[示例](examples/README.md) | [覆盖矩阵](testing/coverage.md)、对应实现面的主规范 |
| 审计历史或变更依据 | `../openspec/changes/` | 按 change 目录读取对应 proposal、design、specs、tasks |

`../openspec/changes/` 用于变更设计、验收和审计历史；日常实现从对应任务主规范进入。

## 交付验证

交付前默认运行：

```bash
bun run verify:docnav-workspace
```

局部验证取舍见 [测试策略](testing.md)，脚本工具链和本地工具运行方式见 [工程工具链](tooling.md)。

## 文档分层

| 类型 | 文档 | 使用时机 |
| --- | --- | --- |
| 项目首页 | [README](../README.md) | 确认项目目标、v0 范围或运行入口 |
| 文档导航 | 本文档 | 定位任务主规范、状态语义、规则 owner 和交付验证入口 |
| 主规范 | [架构](architecture.md)、[CLI](cli.md)、[输出模式](output.md)、[原始协议](protocol.md)、[适配器契约](adapter-contract.md)、[Ref](ref-contract.md)、[测试策略](testing.md) | 修改稳定规则或实现职责 |
| 测试资料 | [测试用例维护](testing/case-maintenance.md)、[测试用例编号账本](testing/cases.md)、[覆盖矩阵](testing/coverage.md)、[发布包验证](testing/release.md) | 测试变更流程、case 条目、覆盖目标或 release 验证 |
| Adapter 专页 | [Markdown Adapter](adapters/markdown.md) | 修改 Markdown adapter 私有行为 |
| 校验材料 | [JSON Schema 索引](schemas/json-schema.md)、[示例](examples/README.md) | 修改字段形状、示例链路或输出映射 |
| 工程规范 | [编码规范](CODING_STYLE.md)、[工程工具链](tooling.md) | 修改代码、脚本或验证工具链 |
| 变更工作流 | `../openspec/changes/`、`../openspec/specs/` | 规划较大 PR、审计 change 设计、验收、归档或历史决策 |

Schema、示例和机器规则文件是验证材料，不在这些文件重新定义产品语义。与主规范不一致时默认修正验证材料；若验证材料代表有意契约变更，必须同步更新 owner 主规范。

## 规范状态与实现状态

Docnav 采用 docs-first 工作流：`docs/` 是长期规范基础；代码、测试和 release artifact 证明当前实现状态。除非明确标注为 Current 或已实现，规范中的 `MUST` / `SHALL` 表示目标契约或决策要求，不自动表示当前二进制已经支持。

状态词只在影响实现或验收判断时使用：

- Current：当前应已支持，并能由代码、测试、验证命令或 release artifact 证明。
- Target / Planned / Historical：分别表示目标、计划或历史背景，不单独证明当前支持。

OpenSpec 用于按 change 规划和审计较大 PR；小功能可以直接修改 docs、代码和测试。冲突时先判断类型：长期方向以 owner 主规范为准；当前支持状态以实现证据为准；同一目标内部冲突必须归并为一个决策；schema、示例或机器规则默认作为验证材料同步。

## 规则所有权

关键规则只由一个主文档拥有，其它文档只摘要或引用，保持规则来源单一。

| 规则面 | Owner 文档 |
| --- | --- |
| 制品职责、调用链、adapter 选择、进程边界 | [架构](architecture.md) |
| `docnav` 命令、`config` 命令入口、adapter 管理命令、adapter 直接 CLI 入口和退出码 | [CLI](cli.md) |
| 标准参数身份、入口字段映射、配置字段映射、来源标记、合并顺序、默认值、透传规则、validation 和 operation binding | [标准参数](standard-parameters.md) |
| 输出模式、readable-view framing、readable-json warning、阅读文案配置、输出通道 | [输出模式](output.md) |
| invoke envelope、operation、紧凑结果、page、稳定错误 | [原始协议](protocol.md) |
| adapter 命令、manifest、probe、格式默认值、invoke 行为 | [适配器契约](adapter-contract.md) |
| ref 的共享调用流程、非空 opaque string、原样传递和 adapter 所有权 | [Ref](ref-contract.md) |
| Markdown ref grammar、结构快照语义、错误分类和显示职责 | [Markdown Adapter](adapters/markdown.md) |
| 自动化测试层级、覆盖目标、一致性审计和 release 验证边界 | [测试策略](testing.md)、[覆盖矩阵](testing/coverage.md)、[发布包验证](testing/release.md) |
| 测试函数变更时的 case 归属、账本更新和源码 `@case` 标记维护流程 | [测试用例维护](testing/case-maintenance.md) |
| 最终 case 条目、证明目标和源码 `@case` 标记映射 | [测试用例编号账本](testing/cases.md) |
| 脚本语言、包管理、TypeScript 脚本运行方式和脚本类型检查验证入口 | [工程工具链](tooling.md) |
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
