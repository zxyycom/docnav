本设计拥有把现有长期决策集合单轨迁移到 decision-records v5 的方案与取舍；审核和实施状态只由 `tasks.md` checkbox 表示，本文件不改变当前决策状态、主规范或验证入口。

## Context

Docnav 当前把 `.codex/skills/decision-records` 作为项目级可复现依赖，`scripts/docs/validate.ts` 直接导入其 ESM 模块，并由 required profile 中的 docs validation 执行严格检查。现有 `docs/decisions` 包含两个已建立记录：

| 路径 | 当前生命周期 | 建立时间 | 直接关系 |
| --- | --- | --- | --- |
| `decision-management/separate-decision-spec-ownership.md` | active | `2026-07-21T07:07:20Z` | `修订` `decision-management/use-verified-decision-records.md` |
| `decision-management/use-verified-decision-records.md` | archived | `2026-07-21T03:51:33Z` | 无 |

旧模型用正文 H1、`索引摘要` 和 `关系` 章节承接索引投影，`decision-index.json` 使用 `schemaVersion: 3` 和顶层 `records`。v5 改为：

- `decision-domains.json` 拥有受控领域；
- 每条 Markdown frontmatter 拥有摘要、生命周期、对齐、建立时间和关系；
- 正文只保留 `目的`、`背景`、`决策`；
- `decision-index.json` 是可删除重建的通用状态索引投影；
- CLI 使用显式 `activate`、`evolve`、`mark-aligned`、`archive` 和 `discard` 事务。

远端 v5 CLI 已对当前工作区做过只读试跑，确认旧集合会因缺少领域表、旧 Markdown 结构和旧索引 shape 而严格失败。迁移只影响项目维护和验证边界，不跨越 Docnav 产品 CLI、adapter、协议或进程边界。

## Goals / Non-Goals

**Goals:**

- 固定采用 release `20260727T030324Z-17ebf93ef2dd` 中 `metadata.version: "5"` 的完整 `decision-records` 分发目录。
- 在不改变现有决策含义、路径、建立时间、生命周期和直接关系的前提下迁移权威 Markdown。
- 为现有 `decision-management` 领域建立显式目录定义，并从权威来源重建新索引。
- 保持仓库内、离线、确定性的 required 校验入口及 owner 分工。
- 用事实核对决定活动记录的初始对齐状态，并留下可审计证据。

**Non-Goals:**

- 不借迁移新增、修订、替代或归档长期判断。
- 不把 OpenSpec change 决策、当前实现状态或任务进度搬入长期决策记录。
- 不依赖个人 skill 安装、运行 updater 或访问网络完成日常验证。
- 不修改 Docnav 产品行为、公共 schema、示例、release artifact 或 adapter contract。

## Decisions

### Decision 1: 固定 v5 release 整包替换

采用已核实 release 中的完整 `decision-records` 目录，包括 `SKILL.md`、references、schema、CLI、类型声明、source map 和 updater。实现不得从浮动 `latest` 拼接文件，也不得只替换 CLI。

这样可以让行为文档、机器 schema 和实际 CLI 保持同一版本。备选方案是只复制 `SKILL.md` 或在项目内维护兼容 fork；前者会造成契约与实现错配，后者会产生第二个维护 owner，因此不采用。

### Decision 2: 逐条保留历史身份并转换权威来源

新增 `docs/decisions/decision-domains.json`，只定义当前已有的 `decision-management` 领域及其稳定责任描述。两个现有路径保持不变，并按以下规则转换：

1. H1 标题、`索引摘要`、旧索引生命周期、建立时间和关系进入 v5 frontmatter。
2. 正文只保留原有 `目的`、`背景`、`决策` 语义；删除重复投影章节，不改写判断。
3. archived 记录使用 `status: archived`、`alignment: null`，保留原 `createdAt`。
4. active 记录保留 `status: active` 和原 `createdAt`。只有完整决策与 `docs/navigation.md`、`docs/tooling.md`、实际 validator 导入路径及 required profile 核对一致后，才写入 `alignment: aligned`；否则使用 `unaligned` 并把差距留在事实 owner 或本 change 任务中。
5. `修订` 关系继续从新记录指向已归档直接前序。

备选方案是把全部 active 记录机械标为 `unaligned`，或根据当前 `active` 机械标为 `aligned`。两者都绕过 v5 对齐语义，因此不采用。

### Decision 3: 新索引只从 v5 权威来源重建

旧 `decision-index.json` 只用于迁移盘点，不进行字段到字段转换。领域表和两条 Markdown 合法后，使用 v5 `sync-index --write` 原子生成新索引，再用 `check` 校验成员、关系、revision 和结构。

新旧 schemaVersion 数字不表达兼容顺序，不能因为新通用索引使用较小版本号而复用旧解析逻辑。不得从旧索引反向补造 Markdown 中没有的含义。

### Decision 4: 单轨切换并保持仓库内验证

Skill、领域表、Markdown、索引、validator 适配和 owner 文档在同一 change 中切换。开发过程中可以依靠 Git 保留旧内容，但最终树中不保留旧 CLI、兼容双读、旧索引解析器或第二份决策源。

`scripts/docs/validate.ts` 继续从项目跟踪路径导入模块，并适配 v5 的公开导出与结果类型；`validate:docs` 的 `decisions` task 和 required profile 角色保持不变。updater 只作为显式维护工具存在，验证链路不得运行它。

### Decision 5: Owner 分工不随存储格式迁移

`docs/navigation.md` 继续拥有 owner 文档、OpenSpec change 和长期决策的分工；`docs/tooling.md` 继续拥有验证入口；v5 skill 拥有通用决策结构和 CLI。新 `decision-record-management` OpenSpec capability 只记录目标契约和验收视图，不成为稳定规则的第二 owner。

### Decision 6: 一次性迁移证据留在 change

实现阶段创建 `migration-record.md`，集中保存实现前审计、旧系统基线、固定上游指纹、活动记录对齐证据、验证结果和完整 changeset 回滚步骤。`tasks.md` checkbox 是执行进度的唯一来源，`migration-record.md` 只承接完成这些任务所需的一次性证据；它不复制目标契约，也不成为长期决策、当前实现状态或验证规则的 owner。

## Risks / Trade-offs

- [错误标记 aligned 会把未核实事实提升为基线] → 为活动记录建立逐项事实核对表；缺少任一关键证据时保持 `unaligned`。
- [格式转换可能改变原决策语义] → 对比迁移前后目的、背景、采用/不采用和直接关系；只移动投影信息，不润色判断。
- [整包替换与数据转换不同步会暂时破坏 required 校验] → 在同一实现 change 中完成单轨切换，并以最终严格检查作为合入门禁。
- [新 CLI API 与 `scripts/docs/validate.ts` 的导入调用不兼容] → 先读取 v5 `.d.mts` 和实际导出，增加最窄 wrapper 适配，不复制 CLI 内部校验。
- [回滚后新格式数据无法被旧工具读取] → 回滚必须恢复整个迁移 changeset，而不是只恢复 skill 或只恢复数据。

## Migration Plan

迁移按四个有明确退出条件的阶段推进；具体进度只在 `tasks.md` 维护：

1. **审计与基线**：完成 artifacts 审计，并在 `migration-record.md` 固定旧系统、目标 release 和允许范围；审计通过前不修改迁移目标。
2. **权威源转换**：整包替换 skill，建立领域表并逐条转换 Markdown；每条记录通过语义保真检查后才进入下一阶段。
3. **单轨切换**：完成活动记录对齐判断、重建索引、适配 validator 并更新 owner 文档；阶段退出时不得存在双读或半迁移状态。
4. **验收与回滚确认**：运行原生 CLI、项目验证和 workspace 验证，记录结果并确认完整 changeset 可整体回滚。

回滚以完整 Git changeset 为单位：同时恢复旧 skill、旧 Markdown、旧索引、validator 和文档。不得让旧工具读取新格式，或让 v5 工具读取半迁移集合。

## Open Questions

无未回答开放问题，可以进入实现前审计。
