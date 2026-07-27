# decision-records v5 迁移记录

本文件保存 `migrate-decision-records-v5` 的一次性审计、基线和执行证据。目标契约由 change spec 拥有，执行进度由 `tasks.md` checkbox 拥有，长期规则和当前实现状态仍由各自 owner 证明。

## 实现前审计

审计日期：`2026-07-27`

结论：通过，可以进入迁移基线盘点。没有未回答开放问题，也没有需要用户选择的实质歧义。

### Artifact 一致性

| 检查面 | 结论 |
| --- | --- |
| 核心目标 | proposal、design、spec 和 tasks 均围绕“保留现有判断语义与身份，单轨迁移到 decision-records v5”。 |
| Capability | `decision-record-management` 是稳定名词短语，表达长期决策管理责任，不包含迁移阶段或版本号。 |
| 临时范围 | OpenSpec artifacts 只位于 `openspec/changes/migrate-decision-records-v5/`；它们不声称主规范或实现已经改变。 |
| 开放问题 | `design.md` 明确无未回答开放问题；现有 artifacts 没有仍待选择的兼容、owner、对齐或回滚方案。 |
| 状态来源 | `tasks.md` checkbox 是执行进度的唯一来源；本记录只保存完成任务所需的证据。 |

### Owner 与上游契约

- `docs/navigation.md` 继续拥有 owner 文档、OpenSpec change 和长期决策的分工、同步与冲突处理。
- `docs/tooling.md` 继续拥有仓库脚本与 required 验证入口。
- `.codex/skills/decision-records/` v5 拥有通用决策结构、生命周期、对齐语义、派生索引和 CLI。
- `docs/decisions/` 的领域表与各条 Markdown 是项目长期决策权威来源；派生索引不反向拥有状态。
- 固定上游 release 为 `20260727T030324Z-17ebf93ef2dd`，source commit 为 `f3d07c5a4be70253b1c28da25830af1d044d4df9`，目标 package 的 `metadata.version` 为 `"5"`。

### 允许范围

- 整包替换 `.codex/skills/decision-records/`。
- 转换 `docs/decisions/` 的领域表、两条现有 Markdown 和派生索引。
- 适配 `scripts/docs/validate.ts` 及其最窄集成证据。
- 更新 `docs/navigation.md`、`docs/tooling.md` 和直接拥有决策维护入口的必要说明。
- 更新本 change 的 tasks 与迁移证据。

### 非目标

- 不改变 Docnav 产品 CLI、adapter、协议、输出、schema、示例或 release contract。
- 不新增、修订、替代或归档现有长期判断。
- 不把 change 决策、迁移进度或当前实现快照写入长期决策正文。
- 不引入旧格式双读、项目内兼容 fork、个人 skill 依赖或联网 required 检查。

### 回滚单位

回滚单位是完整迁移 changeset：v5 skill、领域表、两条 Markdown、派生索引、validator、集成证据和 owner 文档必须一起恢复。不得只回滚其中一部分并留下跨版本工具与数据组合。

## 迁移基线

### 旧 package 与公开 API

- `.codex/skills/decision-records/` 共 11 个文件；`SKILL.md` 没有 `metadata.version`，属于旧的未版本化项目副本。
- 按相对路径排序后对每个文件计算 SHA-256，再对清单计算的 package 指纹为 `2554c85a961e8709f4f582edeebe84c20883d651b373e3d883425435f233b9c1`。
- 公开运行时导出是 `runDecisionRecordsCli`、`scanDecisionRecords` 和 `validateDecisionRecords`；`scripts/docs/validate.ts` 当前只调用 `runDecisionRecordsCli(["check", "--root", process.cwd()])`。

### 旧数据与 CLI 结果

旧 `decision-index.json` 使用 `schemaVersion: 3` 和顶层 `records`。迁移前严格 `check` 成功，摘要为 `1 areas, 2 decisions, 1 active, 1 archived`。

| 路径 | 状态 | 建立时间 | 摘要 | 直接关系 |
| --- | --- | --- | --- | --- |
| `decision-management/separate-decision-spec-ownership.md` | `active` | `2026-07-21T07:07:20Z` | 按作用域分离 owner 文档、OpenSpec change 与长期决策记录，并用仓库内必需校验保护决策集合。 | `修订` `decision-management/use-verified-decision-records.md` |
| `decision-management/use-verified-decision-records.md` | `archived` | `2026-07-21T03:51:33Z` | 版本化管理 docs/decisions，并由 required docs validator 直接调用项目内模块执行严格检查。 | 无 |

- 旧 `list --status all` 返回上述两条记录及各自 title、purpose、background、decision。
- 两次旧 `show` 均返回与当前 Markdown 完全一致的 H1、`索引摘要`、`目的`、`背景`、`决策`，活动记录另外包含 `关系`。
- 旧 `trace` 确认唯一边是活动记录 `--修订-->` 已归档记录。
- 这些结果只用于迁移盘点；v5 frontmatter 和新索引不得从旧索引补造 Markdown 未表达的含义。

### 固定 v5 分发

- 上游 release：`20260727T030324Z-17ebf93ef2dd`。
- Source commit：`f3d07c5a4be70253b1c28da25830af1d044d4df9`。
- `SKILL.md` metadata：`version: "5"`。
- 目标 package 共 11 个文件，目录成员与旧 package 类别相同：`SKILL.md`、agent metadata、三份 references、decision CLI 的 `.mjs`/`.d.mts`/source map、updater 的 `.mjs`/`.d.mts`/source map。
- 目标 package 清单指纹为 `2fc0fe78888c80269cb4ddb8513f7e20d4c38ccf0e1bbb5e1f0224a5c72d39ea`。
- Updater 固定配置为 skill `decision-records`、repository `zxyycom/skills`、source path `skills/decision-records`、release asset `decision-records.zip` 和 manifest asset `skill-release-manifest.json`；它允许显式 `--release-tag`，但不进入项目验证链路。
- 实现只从上述已核实 source commit 的完整 `skills/decision-records/` 目录复制内容；不会在迁移时查询或拼接浮动 `latest`。
- 整包替换后项目目录仍为 11 个文件，清单指纹为 `2fc0fe78888c80269cb4ddb8513f7e20d4c38ccf0e1bbb5e1f0224a5c72d39ea`；`diff -qr` 对固定 source 目录无输出，证明项目副本逐文件一致。

## 对齐证据

活动记录 `decision-management/separate-decision-spec-ownership.md` 建立为 `active + aligned`。逐项依据：

| 决策方向 | 当前事实来源 | 核对结果 |
| --- | --- | --- |
| `docs/` owner 文档拥有稳定规则，代码、测试和 release artifact 证明当前状态。 | `docs/navigation.md` 的“长期决策与 OpenSpec 分工”和“规范状态与实现状态”。 | 一致；文档明确区分稳定规则、change 判断、决策理由和当前实现证据。 |
| Active OpenSpec change 只拥有当前 change 的设计、决策、任务和验收。 | `docs/navigation.md` 的载体所有权表及同步顺序。 | 一致；change 不成为稳定规则或当前状态 owner。 |
| 长期决策只保存跨 change 仍有效的目的、背景、采用方向和演进关系。 | `docs/navigation.md` 的长期决策进入门槛、冲突处理与规则 owner。 | 一致；两条决策正文没有任务进度或当前实现快照。 |
| 决策集合进入仓库内、确定性且离线的 required 验证。 | `docs/tooling.md`、`scripts/docs/validate.ts`、`scripts/docnav-workspace/checks/definitions.ts`。 | 一致；validator 从项目 `.codex/skills` 导入 `runDecisionRecordsCli`，执行 `check`，required profile 运行 `bun run validate:docs`，链路不调用 updater 或网络。 |
| 具体命令和模块路径由工具链 owner 演进。 | `docs/tooling.md` 和当前 v5 `.d.mts`。 | 一致；工具链文档拥有导入路径，v5 保持 `runDecisionRecordsCli` 公开导出。 |
| `openspec/specs/` 不拥有全局决策生命周期。 | `docs/navigation.md` 的分工说明。 | 一致；全局状态由 `docs/decisions` 权威 Markdown 与派生索引承接。 |

全部长期方向已由当前 owner 文档和实际验证链路证明，没有未交付事实差距，因此迁移将活动记录标记为 `aligned`。该判断不依赖旧索引状态或 Git 工作区状态。

### Markdown 语义保真

- 已归档记录的 `目的`、`背景` 和 `决策` 正文与迁移前逐行一致；旧 H1、索引摘要和生命周期投影移入 frontmatter，关系仍为空。
- 活动记录的 `目的`、`背景` 和 `决策` 正文与迁移前逐行一致；旧 H1、索引摘要和 `修订` 关系移入 frontmatter。
- v5 严格检查在索引重建前只报告旧索引 shape、成员缺失和陈旧，没有报告领域、Markdown、生命周期、对齐或关系错误。
- 删除旧 schema v3 索引后，v5 `sync-index --write` 从领域表和两条 Markdown 生成 schema v2 通用索引；随后严格 `check` 通过并报告 `1 domains, 2 decisions, 1 active, 1 aligned, 0 unaligned, 1 archived`。

## Validator 集成证据

- v5 `.d.mts` 继续公开 `runDecisionRecordsCli`，参数仍接受字符串数组；`scripts/docs/validate.ts` 现有 wrapper 已是最小兼容实现，无需改写或增加第二层校验。
- `bun run validate:docs -- decisions` 通过，证明项目 wrapper 从仓库跟踪路径执行 v5 `check` 并正确映射成功退出码。
- Workspace verifier 的成功摘要过滤已从旧 `areas` 格式更新为 v5 的 `domains`、`aligned` 和 `unaligned` 计数。Focused test 先因旧正则无法过滤新摘要而失败，更新后 `scripts/docnav-workspace/verify.test.ts` 为 19 passed、0 failed。
- 使用只读临时目录执行 v5 `check`：合法集合返回 0；未知领域、非法 Markdown、缺失关系目标、`archived + aligned` 非法组合和陈旧索引分别返回 1，并给出对应阻断诊断。
- 上述临时检查直接复用 v5 public CLI，不在项目内复制其领域、Markdown、关系、对齐或索引规则。

## 验证结果与回滚步骤

### 验证结果

| 验证面 | 结果 |
| --- | --- |
| v5 原生命令 | `domains`、`list --status all --alignment all --full-time`、两条 `show`、活动记录 `trace`、`sync-index` dry check 和严格 `check` 全部通过；路径、时间、生命周期、对齐与关系符合迁移基线。 |
| Docs 集成 | `bun run validate:docs -- decisions` 通过，报告 1 个领域、2 条决策、1 条 active aligned 和 1 条 archived。 |
| Focused test | `bun test scripts/docnav-workspace/verify.test.ts` 为 19 passed、0 failed；已证明 v5 成功摘要在组合失败输出中被过滤。 |
| 脚本静态检查 | `bun run typecheck:scripts` 和 `bun run lint:scripts` 通过。 |
| Workspace | `bun run verify:docnav-workspace` 完成 14 个 checks：13 passed、1 warning、0 failed。warning 来自 2 条既有 function-density 观测，均为 0 changed、0 regressions。 |
| OpenSpec 与 Markdown | change 严格验证、全量 OpenSpec 验证、目标 Markdown outline 和 `git diff --check` 在最终 task 更新后重新运行。 |
| 单轨审计 | 活跃代码和稳定文档中没有旧 schema v3 `records`、`索引摘要`/正文关系投影、旧 `areas` 成功摘要、个人 skill 路径或 updater 验证依赖。 |
| Package 一致性 | `.codex/skills/decision-records/` 与固定 release source 目录的 `diff -qr` 无输出。 |

### 完整回滚步骤

优先回滚包含本迁移全部文件的单一 Git changeset。若在尚未形成迁移提交时手工回滚，必须以第一版规划检查点 `f6ed40facb1b18ad3e1692d3f6517893739292a9` 为来源一次恢复以下集合：

1. `.codex/skills/decision-records/` 完整目录。
2. `docs/decisions/decision-management/` 两条 Markdown 与 `docs/decisions/decision-index.json`，并删除迁移新增的 `docs/decisions/decision-domains.json`。
3. `docs/navigation.md` 与 `docs/tooling.md`。
4. `scripts/docnav-workspace/checks/definitions.ts` 与 `scripts/docnav-workspace/verify.test.ts`；`scripts/docs/validate.ts` 本次没有内容变化。
5. 本 change 的 `tasks.md`，并删除一次性 `migration-record.md`。

恢复后使用旧 package 的 `check`、`bun run validate:docs -- decisions` 和相关 script test 验证旧工具与旧数据重新匹配。不得只恢复 skill、只恢复数据或只恢复索引。
