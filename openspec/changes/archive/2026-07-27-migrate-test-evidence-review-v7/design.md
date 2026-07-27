本设计拥有把集中式 case 账本单轨迁移到 test-evidence-review v7 的方案与取舍；审核和实施状态只由 `tasks.md` checkbox 表示，本文件不改变当前测试、主规范或验证结果。

## Context

Docnav 当前由 `docs/testing/cases.md` 集中保存稳定语义 case，并通过 `Status`、`Code` 和源码 `@case` marker 连接主要验证入口。`scripts/tools/validators/case-catalog/` 校验账本、marker、路径和 planned/implemented 状态，`bun run validate:docs -- cases` 进入 required 验证链。

当前只读盘点显示集中账本有 100 个 case 标题、100 个 `Status: implemented` 和 100 个源码 `@case` marker。数量相等只证明旧 validator 的一 case 一 marker 映射完整，不证明 marker 所在位置就是 runner 能独立选择和单独报告的最小原生测试入口。v7 要求每个保留的 runner 最小原生测试入口恰好对应一个 case，且：

- 固定使用 `docs/test-evidence/`；
- `test-evidence-topics.json` 拥有受控 topic；
- 每个 `<topic>/<slug>.md` 只保存一个 case；
- case 只包含 `Entry`、`Contract` 和 `Proves`；
- 不使用 `Status`、角色、源码 marker、自动采集或聚合 case；
- `test-evidence-index.json` 是可删除重建的查询投影。

因此旧 case 不能机械拆文件。迁移需要同时审查真实 runner 报告粒度、owner 契约、可观察证明信号、active change 引用和验证脚本。该变化只影响仓库测试证据与工程验证，不改变 Docnav 产品进程、CLI、adapter、协议或输出边界。

## Goals / Non-Goals

**Goals:**

- 固定采用 release `20260727T030324Z-17ebf93ef2dd` 中 `metadata.version: "7"` 的完整 `test-evidence-review` 分发目录。
- 为本次迁移范围内每个保留的最小原生测试入口建立且只建立一个可查询 case。
- 用受控 topic 和独立 Markdown 替代集中账本、marker 采集与旧派生规则。
- 保留仍成立的 case ID、契约背景和证明语义，并为拆分、删除或转交的旧 case 建立可审计映射。
- 保持项目主规范是产品与测试行为 owner，测试证据只承接压缩背景与可观察证明。

**Non-Goals:**

- 不把 lint、schema、生成物一致性、依赖检查、CI job 或 workspace profile 本身登记为测试 case。
- 不因历史回归、旧 case 文案或当前实现细节新增产品契约。
- 不要求所有 helper、fixture、断言或 supporting test 单独登记。
- 不修改已归档 OpenSpec change 的历史文本。
- 不改变 Docnav 产品行为；仅在旧测试入口混合多个可独立报告意图时拆分测试实现。

## Decisions

### Decision 1: 固定 v7 release 并使用项目级完整分发

新增已核实 release 中完整的 `.codex/skills/test-evidence-review`，包括行为文档、目录契约、migration references、CLI、schema、类型声明和 updater。项目验证直接使用仓库跟踪内容，不依赖个人安装或浮动 `latest`。

备选方案是在现有 case-catalog validator 上逐步模拟 v7；这会长期维护两套契约和机器 schema，因此不采用。

### Decision 2: 先建立显式迁移映射，再转换 case

实现阶段在本 change 目录维护一次性 `migration-map.md`，逐条记录：

- artifacts 审计结论、允许范围、旧系统基线和完整 changeset 回滚步骤；
- 旧 case ID、标题、状态和现有 Code/marker；
- runner 实际最小原生测试入口；
- owner 契约和直接证明信号；
- 目标 topic、目标文件和保留/拆分/删除/转交结果；
- 拆分时旧 ID 的承接入口和新增 ID。

只有旧 case 与一个保留入口语义一一对应时才原样保留 ID。旧 case 聚合多个可独立报告入口时，语义最连续且完整承接原标题/契约的入口可以保留旧 ID，其余入口分配新 ID；不存在自然承接者时，旧 ID 只留在迁移映射中，全部目标入口使用新 ID。planned case 不进入新目录，应回到相关 owner 文档、active change 或待办；supporting helper 和工程 check 不转换为 case。

不采用从源码 marker、文件名或旧 Proves 自动生成目标目录，因为工具无法可靠判断 runner 粒度和契约质量。`tasks.md` checkbox 是执行进度的唯一来源；`migration-map.md` 只承接本次迁移证据，不成为测试行为、case 契约或长期维护规则的 owner。

### Decision 3: Topic 按稳定测试责任而非旧文件形状确定

受控 topic 以当前 owner surface 和稳定维护责任为依据，例如 core CLI/navigation、adapter、protocol/output、diagnostics、shared tooling、release 和 test infrastructure。最终 ID/描述在迁移映射完成后一次确定、排序并写入 `test-evidence-topics.json`。

旧 `BB`、`WB`、`AUX` 类别和 case ID 中的 token 可以辅助定位，但不自动成为 topic；topic 也不改变 case ID 或粒度。这样避免把测试层级、目录布局和责任 owner 混成同一维度。

### Decision 4: 固定目录单轨切换，不提供双读

由于旧账本位于 `docs/testing/cases.md`，实现可以在不覆盖旧源的情况下准备完整 `docs/test-evidence/`。切换前旧账本仍是唯一 owner；新目录通过 topic、index、case 内容和代表性查询检查后，在同一 changeset 中：

1. 停止旧账本写入并删除 `docs/testing/cases.md`；
2. 移除源码 `@case` marker；
3. 删除旧 case-catalog validator 和 marker 采集逻辑；
4. 切换稳定文档、AGENTS、active changes 和验证入口；
5. 让新目录成为唯一权威源。

最终实现不保留兼容双读、`.test-evidence.json`、旧状态字段或自动注册。已归档 OpenSpec change 保持历史原文，不作为活跃调用方。

### Decision 5: 保持 `validate:docs -- cases` 的项目入口角色

为减少无关 package/workspace 调度变化，保留现有 `validate:docs` 的 `cases` task ID，但把实现替换为对项目级 v7 模块的严格 `check`。文档同时公开 v7 原生命令用于 `topics`、`sync-index`、`list` 和 `show`。

旧 `scripts/tools/validators/case-catalog/` 在切换后删除；项目 wrapper 只负责把 v7 结构化诊断接入现有 docs validation，不复制 topic、case 或 index 规则。

### Decision 6: 测试证据只登记原生入口和 owner-backed 证明

每个保留入口按 runner 能稳定选择并单独报告的最小节点判断。文件、suite、package script 和 CI job 只作为容器；fixture、helper、mock、hook、断言和步骤归入所属入口。脚本工具的真实测试函数可以登记，但工程校验命令本身不能登记。

`Contract` 必须来自现有 owner 明文语义，或来自已经确认需要保持的当前行为；`Proves` 只描述直接可观察结果。历史事故只能影响输入选择，不能独立制造断言。

### Decision 7: 稳定资料和 active changes 同步，历史归档不改写

迁移更新 `docs/navigation.md`、`docs/testing.md`、`docs/testing/case-maintenance.md`、`docs/testing/coverage.md`、AGENTS 和当前仍在执行的 change 中对旧账本/marker 流程的要求。归档 changes 只保存当时历史，不批量改写。

## Risks / Trade-offs

- [100 个旧 case 与真实原生入口数量不同，迁移规模可能扩大] → 先建立完整映射并按 topic 分批审查；数量只作为盘点，不作为完成目标。
- [机械保留 ID 会把聚合语义错误绑定到单个入口] → 只有一一对应或存在明确语义承接者时保留；其他拆分显式分配新 ID。
- [删除 marker 后失去自动完整性检查] → v7 明确只保证显式目录完整性；本次范围完整性由迁移映射和审计证明，后续由测试变更流程维护。
- [集中账本拆成大量文件增加文件数量] → 以局部读取、topic 查询和单 case owner 换取更小审查范围；统一索引提供检索。
- [active changes 继续要求旧流程] → 切换前搜索并更新所有非 archive change；归档历史明确排除。
- [测试拆分可能改变运行时序或 fixture 生命周期] → 仅在 runner 意图确实混合时拆分，运行原目标测试和更高层验证比较结果。

## Migration Plan

迁移按四个有明确退出条件的阶段推进；具体进度只在 `tasks.md` 维护：

1. **审计与映射**：完成 artifacts 审计，在 `migration-map.md` 固定旧系统基线，并为每个旧 case 得到唯一迁移结论；映射完整前不创建最终目录。
2. **权威目录建立**：整包新增 v7 skill，建立受控 topic 和逐 case Markdown，必要时按已审阅映射拆分测试入口；目录严格检查通过后才切换。
3. **单轨切换**：生成索引，移除旧账本、marker 和 validator，并同步 wrapper、稳定文档、AGENTS 和非 archive active changes；阶段退出时不得存在双读或双 owner。
4. **验收与回滚确认**：运行受影响测试、原生目录命令、项目验证和 workspace 验证，将结果与最终目录逐项对账并确认完整 changeset 可整体回滚。

回滚以整个迁移 changeset 为单位，同时恢复旧账本、markers、validator、文档和验证入口。不得让旧 validator 检查新目录，或让 v7 工具与旧账本并行作为 owner。

## Open Questions

无未回答开放问题，可以进入实现前审计。
