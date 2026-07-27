本任务清单拥有 decision-records v5 迁移的执行顺序和进度；checkbox 未勾选即表示对应工作未完成，阻塞级审计完成前不得执行任何实现任务。

执行进度只由本文件 checkbox 表示。任务 1.2 创建的 `migration-record.md` 保存一次性审计和迁移证据；记录证据不等于完成任务，只有对应验证通过后才能勾选。

## 1. 实现前阻塞审计

- [x] 1.1 审计 proposal、design、`decision-record-management` spec 与本任务清单是否围绕“保留语义地单轨迁移到 v5”这一核心目标，确认 capability ID 是稳定名词、所有 artifact 只位于当前 change、`## Open Questions` 无未回答项；本项完成前不得执行 2.1 及之后任何实现任务。
- [x] 1.2 对照 `docs/navigation.md` 的 owner 分工和 release `20260727T030324Z-17ebf93ef2dd` 的 v5 package，在 `migration-record.md` 记录审计结论、允许迁移范围、非目标和回滚单位；发现实质歧义时停止并修订 artifacts。

## 2. 固定迁移基线与分发版本

前置：任务 1.1 和 1.2 已完成，且 `migration-record.md` 没有阻塞结论。

- [x] 2.1 在 `migration-record.md` 记录当前 `.codex/skills/decision-records` 指纹、公开导出、旧 CLI `list/show/trace/check` 结果，以及两个现有记录的路径、状态、建立时间、摘要和直接关系。
- [x] 2.2 在 `migration-record.md` 完整记录 release 中 `decision-records` 的 `metadata.version: "5"`、source commit、目录成员和 updater 配置，确认实现只使用固定 release 内容而不追随浮动 `latest`。
- [x] 2.3 整包替换 `.codex/skills/decision-records`，并用目录级 diff 证明项目副本与固定 release 分发目录一致。

## 3. 迁移决策权威来源

前置：任务 2.1 至 2.3 已完成，且项目 skill 与固定 release 目录级一致。

- [x] 3.1 新建按 ID 排序的 `docs/decisions/decision-domains.json`，定义 `decision-management` 的稳定责任描述，并用 v5 `domains` 命令验证。
- [x] 3.2 把 `decision-management/use-verified-decision-records.md` 转换为 v5 frontmatter 与三段正文，保留 archived 生命周期、null alignment、原建立时间、摘要和无关系状态，并以语义 diff 确认判断未改变。
- [x] 3.3 把 `decision-management/separate-decision-spec-ownership.md` 转换为 v5 frontmatter 与三段正文，保留 active 生命周期、原建立时间和指向直接前序的 `修订` 关系，并以语义 diff 确认判断未改变。
- [x] 3.4 将活动决策逐项对照 `docs/navigation.md`、`docs/tooling.md`、`scripts/docs/validate.ts` 和 required profile，在 `migration-record.md` 保存逐项证据；证据完整时写入 `aligned`，否则写入 `unaligned` 并把事实差距交给对应 owner。
- [x] 3.5 删除旧索引投影并运行 v5 `sync-index --write` 重建 `docs/decisions/decision-index.json`，确认没有从旧 schema v3 `records` 反向补造内容。

## 4. 集成仓库内验证与 owner 文档

前置：任务 3.1 至 3.5 已完成，v5 权威来源和派生索引已通过原生严格检查。

- [x] 4.1 按 v5 `.d.mts` 和实际导出适配 `scripts/docs/validate.ts`，保持 `validate:docs` 的 `decisions` task、结构化失败和 required profile 角色，不调用 updater 或网络。
- [x] 4.2 更新或新增最窄 validator 集成证据，证明合法 v5 集合通过，非法领域、Markdown、关系、对齐组合或陈旧索引会阻断。
- [x] 4.3 更新 `docs/navigation.md` 的长期决策分工、决策索引说明和规则 owner，只表达 v5 当前稳定语义，不复制机器 schema。
- [x] 4.4 更新 `docs/tooling.md` 及必要维护说明，记录仓库内 v5 导入路径、离线校验边界、原生命令和 updater 不进入验证链路。

## 5. 验证与单轨切换审计

前置：任务 2.1 至 4.4 已完成；本节只验收完整单轨结果，不用于掩盖前序未完成项。

- [x] 5.1 运行 v5 `domains`、`list`、两个记录的 `show`、活动记录的 `trace`、`sync-index` dry check 和严格 `check`，核对路径、生命周期、建立时间、对齐与关系。
- [x] 5.2 运行 `bun run validate:docs -- decisions`、相关脚本测试、`typecheck:scripts` 和 `lint:scripts`，区分决策数据失败、validator 集成失败和脚本静态失败。
- [x] 5.3 运行 `bun run verify:docnav-workspace`，确认产品 CLI、adapter、协议、输出、schema、示例和 release 验证未被迁移改变。
- [x] 5.4 搜索并确认最终树没有旧索引解析器、兼容双读、个人 skill 依赖或联网更新检查；用局部 diff 证明只迁移目标 skill、决策集合、验证入口和 owner 文档。
- [x] 5.5 在 `migration-record.md` 记录全部验证结果和完整 changeset 回滚步骤，确认回滚会同时恢复旧 skill、旧 Markdown、旧索引、validator 和文档，不产生跨版本半迁移状态。
