# 测试证据维护

本文拥有 Docnav 项目内测试变更、原生入口闭合和测试证据目录的维护流程。通用
NativeTestEntry、Evidence Claim、查询与索引契约由项目级
[`test-evidence-review` skill](../../.codex/skills/test-evidence-review/SKILL.md)
拥有；`scripts/test-evidence/` 拥有本项目的 runner profile、静态规则、runner
调用、入口归一和闭合检查。

权威顺序固定为：

1. 当前源码和 runner 报告拥有入口存在性与 runner 身份。
2. `docs/test-evidence/claims/<topic>/<slug>.md` 拥有长期 Evidence Claim 语义。
3. [`claim-topics.json`](../test-evidence/claim-topics.json) 拥有受控 Claim topic。
4. [`native-test-inventory.json`](../test-evidence/native-test-inventory.json) 和
   `test-evidence-index.json` 是可删除重建的派生制品。

因此 inventory 可以用于离线查询和 Git 审计，但不能创建入口；Claim 可以承接
rename、split 或 merge 后的长期语义，但不能证明一个原生入口仍然存在。

本体系只评估完整当前树。日常检查和查询不读取旧 case、旧 marker、迁移映射或
历史目录；确需审计历史时从 Git 与对应 OpenSpec change 恢复，不把历史材料接回
当前验证链。

## 使用时机

出现以下任一情况时按本文处理：

1. 新增、修改、删除、重命名、移动、拆分或合并原生测试入口。
2. 修改断言、fixture、mock、时序或环境，使证明信号或可靠性发生变化。
3. 新增、修改、删除、查询或审查 Evidence Claim。
4. 修改 supported runner profile、静态规则、runner report 或入口归一逻辑。

只运行既有测试，或只修改被测对象而没有修改测试时，不同步 inventory。只改测试
内部实现且入口身份、证明信号和可靠性都不变时，不新增 Claim；项目严格检查仍会
在完整当前树上确认既有 inventory 和 Claim 没有陈旧。

## 原生入口与 machine case

NativeTestEntry 是 runner 能稳定独立选择或报告，并拥有一项完整测试意图的最小
原生节点。每个 supported Entry 由 project wrapper 生成恰好一个 machine case，
身份是确定性 `entryKey`。Machine case 只投影 runner、target、selector、
source path/range 和 source fingerprint，不使用手写 Markdown 或源码 marker。
Smoke Entry 的 fingerprint 除 leaf task 声明外，还包含其 `run` 绑定在 smoke
`sourceRoots` 内可达的顶层实现声明；同一模块中不可达的其它声明不影响该 Entry。
source roots 外的 fixture、harness 与 assertion 只记录依赖绑定，不作为单个 Entry
独占的实现正文。

判断粒度时依次确认：

1. runner 能否稳定单独命名并报告该节点的通过或失败。
2. 节点是否拥有一项完整、可归因的测试意图。
3. 节点内部是否还有能独立选择或报告的更小原生节点。

Rust 的 `#[test]` 函数、Bun 的 `it` / `test` 和 core smoke profile 展开后的
leaf task 通常是 Entry。以下对象不是 Entry：

- 测试文件、module、suite、package script、runner 和 CI job；
- setup、fixture、helper、mock、hook、断言和测试步骤；
- smoke 聚合 root；
- lint、类型检查、schema、生成物一致性、安全扫描等工程校验。

参数化测试按 runner 的真实报告粒度进入 inventory。一个节点混合多个能独立命名
和失败的意图时，先拆测试节点。静态形态无法可靠归一时必须产生
`unsupported-entry-shape` 并阻断验证，不能靠猜测生成 Entry。

## 全树闭合

版本化 profile 位于
`scripts/test-evidence/supported-runner-profile.json`，明确 Cargo source roots
与 target kinds、Bun source roots 和目录内相对 `include` / `ignore` 规则，以及
smoke factory 和 source roots。常规 Bun 测试由目录规则自动纳入；
`supplementalFiles` 只补充无法归入这些规则的特殊文件，不能重复列出已经匹配的
文件。Bun pattern 必须是相对 source root 的正向 POSIX glob；不接受以 `!` 或
`#` 开头的控制语法。每个 source root 与 supplemental path 都必须词法位于当前
checkout 内，路径各级不得经过符号链接；root 必须是目录，supplement 必须是普通
文件。

展开顺序固定为：分别取得每个 root 的 include match，移除 ignore match，再合并
workspace-relative supplemental files，最后排序去重。静态扫描与 runtime runner
必须复用这一份结果；每个 include 必须有匹配，最终集合不得为空，ignore 可以暂时
无匹配，冗余 supplemental file 会阻断。严格检查始终从完整 profile 执行以下闭合，
不以 Git diff 或旧 inventory 为发现范围：

1. 用项目 ast-grep 规则发现静态候选，并枚举 Cargo、Bun 与 smoke runner report。
2. 规范化两侧身份，双向比较 static 与 runtime 集合。
3. 从闭合集合生成期望 inventory，并双向比较 committed inventory。
4. 校验 Claim topic、owner、`supportedBy` 和派生 index revision。

`static-only`、`runtime-only`、`duplicate-entry` 和
`unsupported-entry-shape` 证明发现链没有闭合；`missing-case`、`orphan-case`
和陈旧 revision 证明 committed inventory 没有闭合。任一项都阻断严格检查。
相对明确 baseline 的 `changes` 报告只用于缩小 AI 对变化入口和 Claim 的审查范围，
不能替代完整当前树检查。

## Evidence Claim

普通 Entry 可以没有 Claim。只有一项长期判断同时满足以下条件时才创建或保留
Claim：

1. `ownerRef` 精确定位当前 owner 文档中的 requirement。
2. `statement` 表达不能由 owner 加 Entry 名称直接恢复的稳定语义。
3. `observations` 描述失败时调用方可判断的结果。
4. `supportedBy` 至少引用一个当前 `entryKey`。
5. 该说明能实质改善后续审查，而不是填充字段。

每个 `claims/<topic>/<slug>.md` 恰好保存一个 Claim。下面代码块只定义字段顺序；
`<...>` 是必须替换的占位符，替换前不是有效 Claim。当前可提交实例见
[required-argument Claim](../test-evidence/claims/core-cli/required-argument.md)。

```markdown
# Claim <CLAIM-ID>: <title>

Topic: `<topic-from-claim-topics>`
Owner ref: `<docs/path.md#requirement-heading>`

Statement:
- <stable contract statement>

Observations:
- <caller-visible result when the contract holds or fails>

Supported by:
- `<current-entry-key>`
```

Claim ID 全局唯一且稳定；topic 必须来自受控表并与目录一致。Claim 不得复述测试名、
AST match、实现步骤或“测试稳定契约”等通用模板，也不得只证明 fixture、mock 或
内部路径。Claim ID 按当前稳定语义命名，不继承旧 case ID；topic 只在至少有一个
当前 Claim 使用时保留。owner 不存在、引用未知 Entry、全部支持入口被删除、owner section 或
关联入口发生未审查变化时，严格检查以 `claim-stale` 或对应诊断阻断。

## 修改流程

1. 按[测试策略](../testing.md)和[覆盖矩阵](coverage.md)确定测试层级、行为 owner
   与需要观察的结果。
2. 用 `topics`、有界 `list` 和 `show` 查询相关 Entry 与 Claim；需要比较时固定
   一份明确的 inventory baseline。
3. 修改测试并运行能独立报告目标 Entry 的最窄 runner 命令。
4. 运行全树 `sync --write`，从当前静态/runtime 闭合集合重建 inventory/index。
5. 用 `changes --baseline` 审查新增、删除、rename candidate、
   `implementation-changed` 与 `claim-stale`；按信息增量门槛更新 Claim。
6. 再运行严格 `check`、目标测试和范围匹配的 workspace verification。

结构变化按长期语义连续性处理：

- **rename**：`entryKey` 可以变化；确认语义连续后保留 Claim ID 并更新
  `supportedBy`。
- **split**：每个新入口独立进入 inventory；按实际证明信号分配旧 Claim，只有
  独立长期判断才拆 Claim。
- **merge**：合并入口不自动合并 Claim；分别判断各长期判断是否仍成立。
- **delete**：删除入口后，所有失去当前支持的 Claim 必须删除、改写或重新关联。

历史事故只作为风险线索或代表性输入来源，不能独立制造 Claim 或断言。planned
行为留在 owner 文档或 active OpenSpec change，不创建没有当前 Entry 支持的 Claim。
自动化需要复制实现、测试专用观测接口或高成本脆弱环境时，在 owner 验证说明或
变更审查中记录 `Manual CR:`，不创建空测试或名义 Claim。

## 查询与验证

从仓库根目录运行：

```bash
bun run test-evidence -- topics --root .
bun run test-evidence -- list --topic <topic> --root .
bun run test-evidence -- show <entry-key-or-claim-id> --root .
bun run test-evidence -- changes --baseline <inventory-path> --root .
bun run test-evidence -- sync --write --root .
bun run test-evidence -- check --root .
```

`--root` 的边界按命令类型区分：

- `topics`、`list` 和 `show` 只读取指定工作区的 evidence 目录，不执行项目 runner。
- `check`、`sync` 和 `changes` 会执行本 checkout 内的项目 runner adapter；它们的
  `--root` 必须指向当前 checkout。其它 checkout 会得到阻断性 profile 诊断，不会
  与当前 checkout 的 runner 结果混合。

`list` / `show` 在 index 缺失或陈旧时只构造带 warning 的内存投影，不隐式写回。
只有 `sync --write` 可以重建 inventory/index。它必须能在 committed
inventory/index 已陈旧时从当前完整静态/runtime 闭合集合重建派生状态；工具自测
在重建前只校验这些文件各自的 schema shape，语义新鲜度由写入后紧接的 strict
`check` 证明。修改测试代码时还要运行目标 runner；跨多个验证入口时，再运行
`bun run verify:docnav-workspace:required` 或完整
`bun run verify:docnav-workspace`。
