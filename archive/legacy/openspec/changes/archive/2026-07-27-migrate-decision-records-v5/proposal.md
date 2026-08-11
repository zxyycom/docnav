本 change 准备把 Docnav 的项目级长期决策集合迁移到 decision-records v5；本 proposal 只拥有迁移动机、范围和影响，不证明 change 已审核、已实施，也不改变现有主规范、决策数据或验证行为。

## Why

项目当前跟踪的 decision-records 包和 `docs/decisions` 数据仍使用旧索引与 Markdown 契约，而上游 v5 已改为受控领域、Markdown frontmatter、显式对齐状态和可恢复演进事务。直接覆盖 skill 会让 required 文档校验立即失效，因此需要一次保留现有决策语义与建立时间的受控迁移。

## What Changes

- **BREAKING**：把决策 Markdown、领域目录表和派生索引迁移到 v5 契约；旧 `索引摘要`、正文关系段和 schema v3 `records` 索引不再作为合法输入。
- 将项目级 `.codex/skills/decision-records` 整包升级到 release v5，并继续由仓库内确定性入口承接 required 校验，不依赖个人 skill 或联网更新检查。
- 保留现有决策路径、建立时间、生命周期和直接演进语义；活动记录只有在与当前事实来源完整核对后才建立 `aligned` 基线。
- 同步长期决策 owner 文档、工程工具链说明、验证入口和迁移验证材料。
- 不改变 Docnav CLI、adapter、协议、输出、schema 或 release 产品契约，也不借迁移改写现有长期判断。

## Capabilities

### New Capabilities

- `decision-record-management`: 定义项目级长期决策的领域、稳定身份、生命周期、对齐、演进、派生索引和仓库内严格验证边界。

### Modified Capabilities

- 无。

## Impact

- Skill 与工具：`.codex/skills/decision-records/`、`scripts/docs/validate.ts` 及其测试或类型检查入口。
- 决策数据：`docs/decisions/decision-domains.json`、现有决策 Markdown、重建后的 `decision-index.json`。
- Owner 文档：`docs/navigation.md`、`docs/tooling.md` 以及决策维护说明。
- 外部依赖：使用 `zxyycom/skills` 已发布的 v5 分发包；运行时仍只依赖仓库跟踪的本地文件。
