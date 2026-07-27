本任务清单拥有 test-evidence-review v7 迁移的执行顺序和进度；checkbox 未勾选即表示对应工作未完成，阻塞级审计完成前不得执行任何实现任务。

执行进度只由本文件 checkbox 表示。任务 1.2 创建的 `migration-map.md` 保存一次性审计、逐 case 映射和迁移证据；记录证据不等于完成任务，只有对应验证通过后才能勾选。

## 1. 实现前阻塞审计

- [ ] 1.1 审计 proposal、design、`test-evidence-management` spec 与本任务清单是否围绕“一个保留的最小原生测试入口对应一个独立 case”这一核心目标，确认 capability ID 是稳定名词、所有 artifact 只位于当前 change、`## Open Questions` 无未回答项；本项完成前不得执行 2.1 及之后任何实现任务。
- [ ] 1.2 对照 `docs/testing.md`、`docs/testing/case-maintenance.md`、release `20260727T030324Z-17ebf93ef2dd` 的 v7 package 和两个迁移 reference，在 `migration-map.md` 记录审计结论、允许迁移范围、非目标和回滚单位；发现实质歧义时停止并修订 artifacts。

## 2. 盘点旧账本与真实测试入口

前置：任务 1.1 和 1.2 已完成，且 `migration-map.md` 没有阻塞结论。

- [ ] 2.1 在 `migration-map.md` 固定迁移基线：记录 `docs/testing/cases.md` 的全部 case ID/标题/状态/Code/Contract/Proves、全部源码 `@case` marker、旧 validator 结果、active change 引用和相关测试命令。
- [ ] 2.2 扩充 `migration-map.md`，为每个旧 case 记录 runner 最小原生入口、owner 契约、证明信号、目标 topic/文件、ID 去向和保留/拆分/删除/转交结论。
- [ ] 2.3 逐个审查测试文件、suite、参数化测试和自定义程序的 runner 报告粒度；容器、fixture、helper、mock、hook、断言和步骤不得独立登记。
- [ ] 2.4 把 planned case、工程 check、历史回归文案和缺少 owner 契约的条目分别转交到 owner 文档、active change、普通验证或删除结论，不把它们机械转换为 v7 case。
- [ ] 2.5 根据稳定测试责任而非 `BB/WB/AUX` 或旧文件形状确定最终 topic ID 与描述，检查排序、唯一性、覆盖范围和每个 case 的唯一归属。

## 3. 建立 v7 分发与权威目录

前置：任务 2.1 至 2.5 已完成；每个旧 case 和本次保留入口在 `migration-map.md` 中已有唯一结论。

- [ ] 3.1 完整核对 release 中 `test-evidence-review` 的 `metadata.version: "7"`、source commit、目录成员、schema 和 updater 配置，并整包新增 `.codex/skills/test-evidence-review`。
- [ ] 3.2 创建排序后的 `docs/test-evidence/test-evidence-topics.json`，只为已有 case 的 topic 创建非空直属目录。
- [ ] 3.3 按迁移映射把一一对应的旧 case 转换为独立 Markdown，保留稳定 ID、仍成立的 Contract/Proves，并让全部 Entry 精确定位同一个原生测试入口。
- [ ] 3.4 对旧聚合 case 按原生入口拆分：由语义连续的入口承接旧 ID，其余分配唯一新 ID；没有自然承接者时在迁移映射中终止旧 ID。
- [ ] 3.5 仅在一个原生测试节点确实混合多个可独立命名和失败的意图时拆分测试实现，并运行该目标测试证明 fixture、顺序和可观察结果保持成立。
- [ ] 3.6 运行 v7 `sync-index --write` 生成 `docs/test-evidence/test-evidence-index.json`，确认索引可删除后从 topic 表和全部 case 原子重建。

## 4. 切换验证入口并移除旧模型

前置：任务 3.1 至 3.6 已完成；v7 权威目录和派生索引已通过原生严格检查。

- [ ] 4.1 把 `validate:docs` 的 `cases` task 切换为导入项目级 v7 模块执行严格 `check`，只做现有 docs validation 结果映射，不复制 catalog 规则。
- [ ] 4.2 增加最窄集成证据，证明合法目录通过，未知 topic、非法 case、重复 ID 或陈旧索引会产生阻断诊断；Entry 是否跨越原生入口继续由迁移映射与 agent 审查证明，不伪装成机器校验。
- [ ] 4.3 删除 `docs/testing/cases.md`、旧 `scripts/tools/validators/case-catalog/`、源码 `@case` marker 及不再使用的 marker/状态/自动采集配置，确保没有双读或双权威源。
- [ ] 4.4 更新 `docs/navigation.md`、`docs/testing.md`、`docs/testing/case-maintenance.md`、`docs/testing/coverage.md` 和 AGENTS，改为 v7 触发边界、原生入口粒度、topic/case owner 与验证流程。
- [ ] 4.5 更新当前迁移 change 之外所有非 archive active change 中对旧账本、`@case` 或旧维护流程的执行要求；保留本 change 的迁移说明和 archive 目录中的历史原文。

## 5. 目录、测试与工作区验证

前置：任务 2.1 至 4.5 已完成；本节只验收完整单轨结果，不用于掩盖前序未完成项。

- [ ] 5.1 运行 v7 `topics`、`sync-index --write`、严格 `check`、每个 topic 的代表性 `list` 和代表性 `show`，核对 sourcePath、case ID、Entry、Contract 与 Proves。
- [ ] 5.2 将迁移映射与最终 topic/case 目录逐条对账，确认每个旧 case 和本次保留的原生入口都有唯一结论，且新索引 case 数由实际入口决定而非旧数量决定。
- [ ] 5.3 运行全部受影响目标测试，区分测试实现失败、被测对象失败、fixture/时序变化和测试证据目录失败。
- [ ] 5.4 运行 `bun run validate:docs -- cases`、相关脚本测试、`typecheck:scripts` 和 `lint:scripts`，确认项目 wrapper 与 v7 结构化诊断一致。
- [ ] 5.5 运行 `bun run verify:docnav-workspace`，确认 Docnav 产品 CLI、adapter、协议、输出、schema、示例和 release 验证没有行为变化。
- [ ] 5.6 搜索并确认除本 change 迁移说明与 archive 历史外，稳定文档、AGENTS、代码和其它非 archive active changes 不再依赖旧账本、marker、状态或 case-catalog；用局部 diff 审计只改变测试证据与验证 owner 范围。
- [ ] 5.7 在 `migration-map.md` 记录全部验证结果和完整 changeset 回滚步骤，确认回滚会同时恢复旧账本、markers、validator、文档和验证入口，不产生 v7/旧模型并行状态。
