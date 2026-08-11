# 文档导航

改动前用本文定位任务主规范、规则 owner 和交付验证入口。本文同时拥有规范状态与实现证据的关系，以及长期决策、调查报告与 Change Plan 的项目级内容路由、交接时机和冲突处理；领域规则细节进入对应 owner 文档。

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
| 恢复或维护长期决策 | [项目级 `decision-records` skill](../.codex/skills/decision-records/SKILL.md) 的 `list`、“长期决策、调查报告与 Change Plan 分工” | 用 `domains` 选择责任领域，以 `show` / `trace` 展开相关记录；写入前读取 skill 的领域契约 |
| 创建、查询或推进 Change Plan | [项目级 `change-plan` skill](../.codex/skills/change-plan/SKILL.md)、`../changes/` | 只有用户明确要求时才新建持久 Change；维护既有计划时用 `list` / `show` 恢复 status、stage、assessment、artifacts 和门禁，再按当前任务选择目标 |
| 沉淀或审阅调查报告 | [项目级 `investigation-report` skill](../.codex/skills/investigation-report/SKILL.md) 的 `list`、`investigations/` | 按主题读取报告原文和随附资源；需要当前口径时再与当前事实 owner 综合 |
| 审计切换前变更历史 | [`../archive/legacy/openspec/LEGACY.md`](../archive/legacy/openspec/LEGACY.md) | 需要复核形成时依据时再进入深层 archive；不从未完成目录或 CLI 状态推断当前计划 |

`decisions/` 保存已确认且会持续影响后续工作的长期判断；`investigations/` 保存形成时证据和认识快照；`../changes/` 保存一次 change 的临时计划上下文。三个集合的 JSON 索引或 metadata 都不替代 Markdown owner 和当前实现证据。日常实现仍从对应任务主规范进入。

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
| 调查报告 | [调查索引](investigations/investigation-index.json)、`investigations/<category>/`、可选的 `investigations/_resources/` | 沉淀形成时背景、目的、证据范围、认识边界和必要原始资源 |
| 变更工作流 | `../changes/<change>/` | 用户明确要求持久 Change 时，维护 proposal、design、tasks、Git 基线和生命周期 |

文档入口按用途命名：

- `README.md` 只用于仓库首页和 crate/package 入口。
- 其它目录文档使用能表达用途的名称，例如 contract example、conformance fixture、
  verification 或 migration record。
- Active Change Plan 直接从 `.change-plan.json` 和当前 stage 要求的 `proposal.md`、
  `design.md`、`tasks.md` 进入；需要额外记录时按 verification、migration 或其它
  实际用途命名，不增加无独立 owner 的 `README.md`。
- `../changes/archive/` 保留形成时文件布局，不仅为同步当前命名规则而回写历史。

Schema、示例和机器规则文件是验证材料，不在这些文件重新定义产品语义。与主规范不一致时默认修正验证材料；若验证材料代表有意契约变更，必须同步更新 owner 主规范。Change Plan 的 `Affected Owners` 直接引用本节表格指向的稳定 owner；稳定语义只在对应 owner 文档中完整维护。

## 规范状态与实现状态

Docnav 采用 docs-first 工作流：`docs/` owner 文档承接当前稳定规范，代码、测试和 release artifact 证明当前实现状态；活动未对齐决策承接已经确认的未来方向，调查报告承接明确要求沉淀的形成时认识，Change Plan 承接被当前任务选择的一次 change 的临时协调上下文。只有满足 skill 门禁且获得当前授权的 plan/implementation 工作才能指导实施；draft、shelved 或仅因目录存在而被列出的 Change 都不是执行队列。

`MUST` / `SHALL` 只有在对应内容标注为 Current 或已实现，并且存在实现证据时，才表示当前二进制能力。

状态词只在影响实现或验收判断时使用：

- Current：当前应已支持，并能由代码、测试、验证命令或 release artifact 证明。
- Target / Planned：目标或计划上下文；跨 change 仍有长期影响的完整方向由活动决策承接。
- Historical：只表示形成时背景，不作为当前规则或未来方向。

Change Plan、调查报告和长期决策记录都不作为当前实现证据；它们与 owner 文档的分工和同步顺序见“长期决策、调查报告与 Change Plan 分工”。已经明确要求直接完成的局部改动可以直接修改 docs、代码和测试，不为仪式性留档创建计划。

## 长期决策、调查报告与 Change Plan 分工

先按内容含义选择 owner，再按对应 skill 的触发条件决定是否创建或维护载体；内容归属不自动产生写入授权、任务或生命周期动作。

| 载体 | 完整拥有 | 不作为 |
| --- | --- | --- |
| `docs/` owner 文档 | 已成为当前基线的稳定行为、public contract、职责边界和验证语义。 | 当前实现已经成立的单独证据。 |
| `docs/decisions/` | 已确认且跨 change 仍有长期影响的方向、理由、约束和演进关系。 | 任务进度、实现快照、当前任务授权或 change-local 方案。 |
| `docs/investigations/` | 明确要求沉淀的特定时点背景、目的、证据范围、认识边界和必要原始资源。 | 当前需求、已采用决定、累积当前口径或实施计划。 |
| `changes/<change>/` | 被当前任务选中后，该次 change 的目标、范围、change-local 设计、任务、验证、Git 基线和生命周期。 | 稳定事实 owner、跨 change 长期方向、形成时证据 owner、优先级或实施授权。 |

### 创建门槛

1. 当前行为、public contract 或稳定职责发生变化时，更新对应 owner 文档；是否已经实现仍由代码、测试和 release artifact 证明。
2. 判断具有跨 change 长期影响、已经完整到值得审核且当前任务授权维护决策时，交给 `decision-records`；尚未完整的判断可以暂存在当前对话或所选 Draft，但不能带入已确认 plan 继续充当隐藏决定。
3. 只有用户明确要求记录、沉淀、维护或审阅调查时，才创建、追加或修正调查报告；普通调查结果不自动进入 `docs/investigations/`。
4. Change Plan 的创建与写入触发以项目级 `change-plan` skill 为准，Docnav 不增加更宽的例外：跨文件、跨 owner、跨验证阶段或需要对话外交接只能触发“建议创建”的提醒；用户已经明确要求直接完成的工作不为留档另建计划。

### 交接与冲突

1. 确认 plan 前，已经确认且跨 change 持续有效的判断必须交给决策 owner，当前事实必须从稳定 owner 读取；调查报告只能提供证据，不能替代两者。只影响本 change 的人工批准、证据或依赖门禁可以保留在 plan 中，但必须写明 owner、关闭动作和被阻塞的后续任务。
2. 已确认的长期决定由 Change Plan 按引用作为输入；计划不复制完整理由或重新充当第二份 spec，只负责把方向落实为本 change 的范围、change-local 设计、门禁、任务和验证。计划可以在进入 implementation 后先执行显式门禁，但门禁关闭前不得执行其阻塞的 owner、测试或代码修改。
3. 实施中只影响当前 change 的选择留在 plan；新形成的长期方向在确认时交给决策 owner，已经成为当前事实的结果同步到 owner 文档和实现证据。
4. Change 完成时核对 owner、实现、验证和相关决策的实际关系，再按对应 skill 处理 alignment 与归档；完成动作不用于首次补记早已阻塞实施的决定。
5. 载体不一致时分别处理：当前规范与实现证据之间按一致性问题修正当前载体；决策按其对齐与演进语义维护；Change Plan 只对当前任务选中的 change 生效，并继续受 stage、assessment 和授权约束；调查报告保留形成时认识，除非明确维护报告，否则不为追平当前事实而改写；归档历史不参与同步。
6. 决策、调查和 Change Plan 的结构、合法状态、生命周期与维护命令分别由对应项目 skill 拥有；本文只拥有项目级内容路由、交接时机和冲突处理，不复制 skill 契约。

## 规则所有权

关键规则只由一个 owner 完整解释。按下表选择下一份文档；表中的规则面只描述稳定责任边界，不列 owner 内部的功能清单。

| 规则面 | Owner 文档 |
| --- | --- |
| 规范状态与实现证据的关系，以及长期决策、调查报告、Change Plan 与历史材料之间的项目级内容路由、交接和冲突处理 | 本文档 |
| 项目级长期决策的领域划分、内容、对齐、演进和维护 | [决策领域表](decisions/decision-domains.json)与各条决策 Markdown；领域划分遵循[按主要被改变契约组织决策领域](decisions/decision-management/organize-decision-domains-by-primary-changed-contract.md)，通用结构和维护动作由[项目级 `decision-records` skill](../.codex/skills/decision-records/SKILL.md)拥有 |
| Change Plan 的创建门槛、artifact、状态、阶段、Git 基线和生命周期 | [项目级 `change-plan` skill](../.codex/skills/change-plan/SKILL.md)；本文只拥有 Change Plan 与项目稳定 owner 的分工 |
| 调查报告的创建门槛、形成时快照、随附资源、索引、状态和维护 | [项目级 `investigation-report` skill](../.codex/skills/investigation-report/SKILL.md)；各主题 Markdown 与资源拥有形成时内容 |
| 组件与共享 crate 职责、输出分层、调用链和运行边界 | [架构](architecture.md) |
| 跨格式 adapter library contract、registry-facing definition、manifest、共享 operation result 和 adapter-owned private state 边界 | [适配器契约](adapter-contract.md) |
| CLI surface 以及 core-owned command、argv/help、项目根与路径、配置、inspection、logging 和退出行为 | [CLI](cli.md) |
| Navigation command 的输入来源解析、adapter selection、request construction、adapter document 编排、组合和 dispatch | [Navigation Input Resolution](navigation-input-resolution.md) |
| Public output mode、renderer 编排、readable-view 表示和输出通道 | [输出模式](output.md) |
| Protocol request/response envelope、operation result、pagination、failure envelope 和 protocol error projection | [原始协议](protocol.md) |
| 跨格式 ref producer/consumer、兼容文档视图、opaque 传递、成功保证和责任分层 | [Ref](ref-contract.md) |
| Markdown 格式的解析、导航、ref、错误和显示边界 | [Markdown Adapter](adapters/markdown.md) |
| JSON 格式的解析、导航、ref、错误和显示边界 | [JSON Adapter](adapters/json.md) |
| 自动化测试层级、覆盖目标、一致性审计和 release 验证边界 | [测试策略](testing.md)、[覆盖矩阵](testing/coverage.md)、[发布包验证](testing/release.md) |
| Semantic Case 与当前测试实体的关系、Case 语义和账本维护规则 | [语义测试 Case 维护](testing/case-maintenance.md)拥有稳定规则，`testing/cases/<topic>.md` 拥有当前 Case 语义与实体映射；`../scripts/test-evidence/` 实现项目检查，项目级 [`test-evidence-review` skill](../.codex/skills/test-evidence-review/SKILL.md)提供通用评审方法 |
| 当前测试实体的存在性和 runner 身份 | 当前源码和 runner 报告；project wrapper 只发现、归一并比较当前集合 |
| Topic 的受控分类、说明和顺序 | [Case topic 表](testing/cases/topics.json)；Topic 不拥有行为契约 |
| 工具版本、项目环境、包管理、本地工具运行和脚本验证入口 | [工程工具链](tooling.md) |
| JSON 字段形状和契约示例验证 | [JSON Schema 索引](schemas/json-schema.md)、[契约示例](examples/contract-examples.md)；产品语义仍由上方对应 owner 拥有 |

## 术语

本节只保留影响文档权威性和内容路由判断的跨载体术语；产品与实现术语进入上方对应 owner 文档。

| 术语 | 定义 |
| --- | --- |
| owner 文档 | 某类规则的完整解释和维护位置；其它文档只保留摘要或引用。 |
| 长期决策 | `docs/decisions/` 中已经确认且跨 change 持续有效的方向、理由、约束和演进关系；不表示当前实现或任务授权。 |
| investigation report | `docs/investigations/` 主题内形成于特定时点、可独立汇报的认识快照；最新报告不自动等于累积当前口径。 |
| Change Plan | `changes/<change>/` 中被当前任务选中的 change-local 临时计划；stage、assessment 和 checkbox 都不替代当前授权、内容审阅或事实证据。 |
