# 文档导航

改动前用本文定位任务主规范、规则 owner 和交付验证入口；规则细节进入对应 owner 文档。

## 如何阅读这些文档

按任务进入对应主规范；跨模块、边界、状态或规则归属不清时补读 [架构](architecture.md)、“规范状态与实现状态”和“规则所有权”。

| 角色 / 任务 | 必读 | 需要时再读 |
| --- | --- | --- |
| 实现 `docnav` 核心 CLI | [架构](architecture.md)、[CLI](cli.md)、[Navigation Input Resolution](navigation-input-resolution.md)、[输出模式](output.md) | [原始协议](protocol.md)、[适配器契约](adapter-contract.md)、[测试策略](testing.md) |
| 实现原始协议或机器输出 | [原始协议](protocol.md)、[输出模式](output.md) | [JSON Schema 索引](schemas/json-schema.md)、[适配器契约](adapter-contract.md) |
| 实现 Markdown adapter | [适配器契约](adapter-contract.md)、[Ref](ref-contract.md)、[原始协议](protocol.md)、[Markdown Adapter](adapters/markdown.md) | 对应实现面的主规范 |
| 实现 JSON adapter | [适配器契约](adapter-contract.md)、[Ref](ref-contract.md)、[原始协议](protocol.md)、[JSON Adapter](adapters/json.md) | [输出模式](output.md)和对应实现面的主规范 |
| 新增、修改或审查测试 | [测试策略](testing.md)、对应行为 owner、[语义测试 Case 维护](testing/case-maintenance.md) | [覆盖矩阵](testing/coverage.md)、项目级 [`test-evidence-review` skill](../.codex/skills/test-evidence-review/SKILL.md) |
| 修改验证脚本或 workspace check | [工程工具链](tooling.md)、[测试策略的统一验证入口](testing.md#统一验证入口)、[编码规范](coding-style.md) | 变更涉及实体发现或 Case 映射时再读[语义测试 Case 维护](testing/case-maintenance.md)；涉及字段或示例时读[JSON Schema 索引](schemas/json-schema.md)和[契约示例](examples/contract-examples.md) |
| 恢复或维护长期决策 | [项目级 `decision-records` skill](../.codex/skills/decision-records/SKILL.md) 的 `list`、“长期决策与 OpenSpec 分工” | 用 `domains` 选择责任领域，以 `show` / `trace` 展开相关记录；写入前读取 skill 的领域契约 |
| 审计历史或变更依据 | `../openspec/changes/` | 按 change 目录读取对应 proposal、design、specs、tasks |

`decisions/` 保存已确认且会持续影响后续工作的长期判断；`decisions/decision-domains.json` 定义受控领域，各条决策 Markdown 拥有自身内容、生命周期、对齐和直接演进关系，[决策索引](decisions/decision-index.json) 只是可删除重建的查询投影。`../openspec/changes/` 用于变更设计、验收和审计历史。日常实现仍从对应任务主规范进入。

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
| 测试资料 | [语义测试 Case 维护](testing/case-maintenance.md)、[Case topic 表](testing/cases/topics.json)、`testing/cases/<topic>.md`、[覆盖矩阵](testing/coverage.md)、[发布包验证](testing/release.md) | 测试变更流程、当前实体与语义 Case 映射、覆盖目标或 release 验证 |
| Adapter 专页 | [Markdown Adapter](adapters/markdown.md)、[JSON Adapter](adapters/json.md) | 修改对应 adapter 的私有行为 |
| 校验材料 | [JSON Schema 索引](schemas/json-schema.md)、[契约示例](examples/contract-examples.md) | 修改字段形状、示例链路或输出映射 |
| 工程规范 | [编码规范](coding-style.md)、[工程工具链](tooling.md) | 修改代码、脚本或验证工具链 |
| 长期决策 | [决策索引](decisions/decision-index.json)、`decisions/<topic>/` | 恢复跨 change 仍有效的目的、背景、采用方向和演进关系 |
| 变更工作流 | `../openspec/changes/`、`../openspec/specs/` | 规划较大 PR，审计 change 设计、验收或归档；capability 命名见下方对照表 |

文档入口按用途命名：

- `README.md` 只用于仓库首页和 crate/package 入口。
- 其它目录文档使用能表达用途的名称，例如 contract example、conformance fixture、
  verification 或 migration record。
- 活动 OpenSpec change 直接从 `proposal.md`、`design.md`、`specs/` 和 `tasks.md`
  进入；需要额外记录时按其 verification、migration 或其它实际用途命名，不增加
  无独立 owner 的 `README.md`。
- `../openspec/changes/archive/` 保留形成时的文件布局，不仅为同步当前命名规则而
  回写历史。

Schema、示例和机器规则文件是验证材料，不在这些文件重新定义产品语义。与主规范不一致时默认修正验证材料；若验证材料代表有意契约变更，必须同步更新 owner 主规范。

OpenSpec capability ID 表示稳定 owner surface，不表示一次性 change name。跨层总览统一进入 `docnav-architecture`；字段、输出、诊断、ref、adapter 和验证规则进入各自 owner，避免把多个 owner 合并成总包 capability。

| OpenSpec capability | 对应 owner | 使用时机 |
| --- | --- | --- |
| `docnav-architecture` | [架构](architecture.md) | 组件职责、调用链、运行边界和跨层不变量 |
| `core-cli` | [CLI](cli.md) | `docnav` 命令、argv、path/config、static registry 和退出行为 |
| `navigation-input-resolution` | [Navigation Input Resolution](navigation-input-resolution.md) | config source、lexical pathname routing sequencing、adapter selection、typed extraction、request construction、AdapterDocument lifecycle 和 dispatch |
| `adapter-contract` | [适配器契约](adapter-contract.md) | linked adapter factory / AdapterDocument interface、manifest pathname hints、selection contract、closed standard input 和 operation result |
| `protocol-contract` | [原始协议](protocol.md) | raw protocol envelope、operation/result pairing、page 和 protocol failure |
| `output-contract` | [输出模式](output.md) | public output modes、统一 `ProtocolResponse` 输入、`ProtocolJson` / `Rendered(RenderStrategy)`、renderer contract 和 output channels |
| `diagnostics-contract` | [架构](architecture.md) | DiagnosticCode、DiagnosticRecord、canonical details 和 primary projection |
| `ref-contract` | [Ref](ref-contract.md) | opaque ref、explicit ref input、adapter-owned grammar、compatible-view round trip 和 producer/read 原样传递流程 |
| `markdown-adapter` | [Markdown Adapter](adapters/markdown.md) | Markdown pathname hints、parser/ref/outline/read/find/info 和 typed adapter input semantics |
| `json-adapter` | [JSON Adapter](adapters/json.md) | JSON pathname hints、private parse model、ref/outline/read/find/info/full-read 和 JSON-owned errors |
| `typed-fields` | [架构](architecture.md) | typed field identity、constraint metadata、schema metadata projection 和 duplicate guard |
| `contract-validation` | [JSON Schema 索引](schemas/json-schema.md)、[契约示例](examples/contract-examples.md) | schema/example validation、runtime validation parity 和 drift checks |
| `release-artifacts` | [发布包验证](testing/release.md) | package layout、manifest/checksum 和 release artifact verification |
| `test-evidence-management` | [语义测试 Case 维护](testing/case-maintenance.md) | supported runner profile、当前测试实体闭合、语义 Case、topic 和有界查询 |
| `repository-quality-observability` | [工程工具链](tooling.md) | 非阻断质量快照、报告、baseline delta 和扫描边界 |
| `openspec-governance` | 本文档 | OpenSpec、长期决策记录与 docs-first 的分工，以及 capability 命名和归档规则 |

## 规范状态与实现状态

Docnav 采用 docs-first 工作流：`docs/` owner 文档承接当前稳定规范，代码、测试和 release artifact 证明当前实现状态；活动未对齐决策承接已经确认的未来方向，OpenSpec change 承接实施计划。

`MUST` / `SHALL` 只有在对应内容标注为 Current 或已实现，并且存在实现证据时，才表示当前二进制能力。

状态词只在影响实现或验收判断时使用：

- Current：当前应已支持，并能由代码、测试、验证命令或 release artifact 证明。
- Target / Planned：目标或计划上下文；跨 change 仍有长期影响的完整方向由活动决策承接。
- Historical：只表示形成时背景，不作为当前规则或未来方向。

OpenSpec change 和长期决策记录都不作为当前实现证据；它们与 owner 文档的分工和同步顺序见“长期决策与 OpenSpec 分工”。小功能可以直接修改 docs、代码和测试。

Manifest pathname routing、route-before-document-I/O、probe deletion、invocation-private adapter document lifecycle 和 compatible-view ref round trip 已由代码与测试证明为 Current；release artifact 继续证明对外协议和 CLI 兼容性。对应长期规则由下方 owner 文档拥有。

## 长期决策与 OpenSpec 分工

当前基线、未来方向和实施计划分层维护；同一判断只由一个 owner 完整解释。

| 载体 | 核心职责 |
| --- | --- |
| `docs/` owner 文档 | 已成为当前基线的稳定行为、public contract、职责边界和验证语义。 |
| `openspec/changes/<change>/` | 服务该 change 当前阶段的探索依据、设计、`## Decisions`、任务、验收依据和审计历史。 |
| `docs/decisions/` | 已确认且跨 change 仍有长期影响的方向、理由、约束和演进关系；对齐状态说明其与当前事实的关系。 |

按以下顺序记录和同步：

1. 只影响一个 active change 的判断写入该 change；跨 change 仍有长期影响的已确认判断写入 `docs/decisions/`；形成当前基线的结果写入对应 owner 文档。
2. Future change 在探索阶段保留足以恢复意图的目标、约束、依据、开放问题和启动条件；进入实施准备后，再形成届时需要的设计、任务和验收依据。
3. 已形成详细 artifacts 的 change 暂停后继续保留其审计上下文；恢复时根据当前基线、活动决策和实现状态更新仍需使用的内容。
4. Change 收敛后，将形成当前基线的结果同步到 owner 文档和实现证据，将改变长期方向的判断同步为决策演进，再完成归档。
5. 载体之间不一致时，当前稳定规则以 owner 文档为准，当前实现以代码、测试和 release artifact 为准，未来方向以活动决策为准，实施计划以 active change 为准；随后同步失配载体。
6. `openspec/specs/` 只作为 capability specification 的 OpenSpec 工具视图；全局决策状态、对齐和关系由各条决策 Markdown 拥有，[决策索引](decisions/decision-index.json) 只提供可重建查询视图。

活动决策已经确认。`aligned` 表示完整方向已与相关当前事实核对并建立为持续基线；`unaligned` 表示已经确认但尚未成为当前事实的未来方向。相关工作把未对齐方向作为未来演进输入，在满足本次任务的可行方案中优先保留通向该方向的路径，并可在当前任务范围已经覆盖时顺手推进。对齐状态说明方向与当前事实的关系；本次任务范围来自当前请求，未来先后关系来自决策正文。已对齐基线后来与事实偏离时按一致性问题处理。

## 规则所有权

关键规则只由一个主文档拥有，其它文档只摘要或引用，保持规则来源单一。

| 规则面 | Owner 文档 |
| --- | --- |
| 长期决策、OpenSpec change 与 owner 规范的分工、同步和冲突处理 | 本文档 |
| 项目级长期决策的领域、内容、生命周期、对齐和直接演进关系 | [决策领域表](decisions/decision-domains.json)与各条决策 Markdown；通用结构和维护动作由[项目级 `decision-records` skill](../.codex/skills/decision-records/SKILL.md)拥有 |
| 组件职责、输出分层、adapter document 在系统中的高层生命周期位置、调用链和运行边界 | [架构](architecture.md) |
| adapter library interface、manifest format identity/pathname hints、fixed public operation、无 I/O factory、private state enclosure、adapter 选择、registry invariant、格式默认值交接边界和 adapter contract 边界 | [适配器契约](adapter-contract.md) |
| `docnav` 命令、项目根解析、lexical routing pathname 与 post-selection document path 规范化、`config` 命令入口、内置 adapter inspection、strict argv parser/help 和退出码 | [CLI](cli.md) |
| navigation command 的 raw command、config source descriptors/paths、core parameter catalog 和 registry 交接、routing 必需输入解析、route-before-document-I/O sequencing、full config validation、adapter selection、selected-operation catalog filtering、explicit/conditional env/project/user/built_in 来源解析、typed-field 校验提取、strict caller input blocking、adapter document 的创建时机与跨 stage 组合复用、protocol/closed adapter/core output projections 和 no-fallback adapter dispatch | [Navigation Input Resolution](navigation-input-resolution.md) |
| public output modes、两条 document output paths 共同消费 `ProtocolResponse` 的编排规则、renderer selection、readable-view framing、阅读文案配置和输出通道 | [输出模式](output.md) |
| protocol envelope、operation、紧凑结果、page、protocol failure envelope、protocol error fields、code/details 规则和 primary diagnostic projection | [原始协议](protocol.md) |
| diagnostic/error model helper crate 边界、typed diagnostic code、record draft/record、details validation 和 projection helper materials | [架构](architecture.md) |
| ref producer/consumer、兼容文档视图、共享调用流程与成功保证、explicit ref input 非空校验、opaque string、原样传递、round-trip consistency、defect 分类和 adapter 所有权 | [Ref](ref-contract.md) |
| Markdown ref grammar、兼容视图 correspondence、结构快照语义、错误分类和显示职责 | [Markdown Adapter](adapters/markdown.md) |
| JSON pathname hints、selected-operation parse、private model、base/direct/tail ref grammar 与兼容视图 correspondence、导航顺序、source-region find、structured/full-read 和 JSON-owned error 边界 | [JSON Adapter](adapters/json.md) |
| 自动化测试层级、strict failure 覆盖目标、primary DiagnosticRecord 投影、一致性审计和 release 验证边界 | [测试策略](testing.md)、[覆盖矩阵](testing/coverage.md)、[发布包验证](testing/release.md) |
| 测试变更时的 Case 粒度、存储/查询、supported runner profile、静态/runtime/Case 映射闭合和项目验证流程 | [语义测试 Case 维护](testing/case-maintenance.md)拥有稳定规则；`../scripts/test-evidence/` 实现项目检查；通用评审方法由[项目级 `test-evidence-review` skill](../.codex/skills/test-evidence-review/SKILL.md)提供 |
| 当前测试实体的存在性与 runner 身份 | 当前源码和 runner 报告；project wrapper 只发现、归一并比较当前集合，不提交派生实体清单 |
| 当前 implemented 测试目的、Owner/Proves 与实体映射 | `testing/cases/<topic>.md` 中的语义 Case；完整维护规则见[语义测试 Case 维护](testing/case-maintenance.md) |
| Case 的受控查询分类、说明和顺序 | [Case topic 表](testing/cases/topics.json)；Topic 不拥有行为契约 |
| 工具版本、项目环境配置与检测、包管理、TypeScript 脚本运行方式和脚本类型检查验证入口 | [工程工具链](tooling.md) |
| typed field definition core 的共享 crate owner、字段事实源、校验归属和 schema metadata view 边界 | [架构](architecture.md) |
| JSON 字段形状和示例语义校验 | [JSON Schema 索引](schemas/json-schema.md)、[契约示例](examples/contract-examples.md) |

## 术语

本节保留影响任务路由和规则 owner 判断的跨文档词；完整规则进入上方 owner 文档。

| 术语 | 定义 |
| --- | --- |
| owner 文档 | 某类规则的完整解释和维护位置；其它文档只保留摘要或引用。 |
| docnav | 核心 CLI，负责格式识别、adapter 路由、配置、管理和输出分发。 |
| adapter | 独立格式处理组件；通过 factory 创建 invocation-private `AdapterDocument`，并拥有格式解析、导航算法、ref 和分页语义。 |
| `AdapterDocument` | Selected adapter 为一个 normalized document path 创建的 invocation-private lifecycle owner；它懒准备并复用 private view，不把 state 暴露给 caller。 |
| document | Docnav 操作的输入文件；caller path 先词法派生 routing pathname 供 adapter 选择，选择后才形成 operation 使用的 normalized document path。 |
| routing pathname | Invocation-private lexical pathname，由 caller path 与 command cwd 派生，只用于 target-document I/O 前的 manifest basename lookup；不进入 adapter input 或 public output。 |
| `outline -> ref -> read` | 标准导航流程：先取结构条目，再把 adapter 生成的 ref 原样传回读取。 |
| ref | adapter 生成和解析的非空 opaque string；共享层只原样传递。 |
| readable output | 面向人类和 AI 的 `readable-view` 文本；CLI 使用内置 renderer，linked caller 可以通过 shared output API 注入自定义 renderer。规则见 [输出模式](output.md)。 |
| protocol output | 面向脚本、调试和兼容校验的稳定 envelope；协议语义见 [原始协议](protocol.md)，CLI 模式见 [输出模式](output.md)。 |
| current test entity（当前测试实体） | 当前源码中能被 supported runner profile 静态发现、并由 runner 报告的可寻址测试节点；存在性和身份来自当前源码与 runner，不来自 committed 清单。 |
| Semantic Case（语义 Case） | `testing/cases/<topic>.md` 中人工维护的当前 implemented 测试目的；通过 `Owner`、`Proves` 和 `Entities` 连接行为契约与当前测试实体，完整规则见[语义测试 Case 维护](testing/case-maintenance.md)。 |
| Topic | 由 [Case topic 表](testing/cases/topics.json)控制的有界查询分类；Topic 分组 Case，但不拥有行为契约，也不替代 Case 的 `Owner`。 |
