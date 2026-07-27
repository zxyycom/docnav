本任务清单把测试证据 v8 迁移拆成可核验的小步；第 1 组是阻塞级审计，全部完成并在 `audit.md` 得到明确的 `Proceed` 结论前，不得执行第 2 组及其后的实现任务。

实施证据固定保存在本 change：`audit.md` 记录观测、固定输入和 gate 结论，`migration-map.json` 记录每个旧 case 的唯一去向，`verification.md` 记录实际运行的命令、结果和环境限制。三者只服务本次迁移审计；完成任务时应链接或更新对应证据，不在多个 artifact 重复完整结论。

## 1. 阻塞级审计

- [ ] 1.1 在 `audit.md` 逐项核对 proposal、design、delta spec 与本 tasks 的核心句、能力归属、breaking 范围、非目标和术语，记录不一致并修正到零；若出现会改变责任边界的开放问题则停止实施。
- [ ] 1.2 在 `audit.md` 重新测量当前 v7 skill、case、topic、派生索引、模板分类和未纳入入口的可复核路径、数量与内容 fingerprint，并逐项解释相对建 change 时 431 case、11 topic、426 个通用 Contract 模板、396 个 Contract/Proves 双模板、81 个 Rust 漏项和 3 个 Bun 漏项的差异。
- [ ] 1.3 固定 `test-evidence-review` v8 与 `ast-grep` skill 的上游仓库、commit/release、完整分发文件清单和内容 fingerprint，确认许可证、更新方式与本地副本 owner；来源或内容不能固定时停止实施。
- [ ] 1.4 固定开发期 ast-grep CLI 的精确版本、仓库声明的 bootstrap/调用入口和 lockfile 归属，证明依赖准备完成后的 required check 不访问网络且该 CLI 不进入 canonical release file set；无法与产品期 `add-ast-grep-code-adapter` 依赖边界隔离时停止实施。
- [ ] 1.5 枚举 supported runner profile 的 Cargo test targets、Bun test surfaces、smoke task roots、对应 list/report 命令、selector 语法和现有验证入口，保存可重放的当前树基线及失败语义。
- [ ] 1.6 在仓库外临时目录用最小正反例验证 Rust、Bun 与 smoke 的静态候选、runner 枚举和身份归一可行性，覆盖宏、alias、wrapper、参数化、动态注册与 task 组合；不能可靠归一的形态必须能稳定成为 `unsupported-entry-shape`。
- [ ] 1.7 审计当前 specs、稳定文档、验证脚本和全部 active changes 对 v7 case、marker、topic/index 与 ast-grep 的依赖，明确每处切换 owner，并确认不会修改 Docnav CLI、adapter、protocol、ref、输出或产品 release 行为。
- [ ] 1.8 固定单轨切换 changeset、完整回滚单位、性能基线、验收命令和实现停工条件；只有 `audit.md` 明确记录“输入已固定、原型可行、冲突可处理、可原子回滚”并给出 `Proceed` 结论后，才允许开始 2.1。

## 2. 固定 Skill 与开发工具链

- [ ] 2.1 按审计 fingerprint 完整接入项目级 `.codex/skills/ast-grep/`，保留其规则编写、JSON 输出、rule test 与更新说明，并验证无文件遗漏。
- [ ] 2.2 按审计 fingerprint 完整接入 `.codex/skills/test-evidence-review/` v8，使通用 skill 只拥有 NativeTestEntry、Evidence Claim、查询/index 契约、审查流程和完成标准。
- [ ] 2.3 用审计选定的仓库声明方式锁定开发期 ast-grep CLI 及 lockfile，并增加离线安装/调用验证；不得依赖个人 PATH、浮动网络版本或 updater。
- [ ] 2.4 建立 Rust、Bun、smoke 的 project rule 与 rule test 目录，为每种受支持入口和最接近的非入口提供正反例，并为已知动态形态提供 unsupported fixture。
- [ ] 2.5 增加工具链边界检查，证明 external ast-grep 只由开发验证 wrapper 调用，canonical release file set 与产品运行时均不包含该 executable 或规则。

## 3. 原生入口发现与闭合检查

- [ ] 3.1 定义版本化 supported runner profile、共享 `NativeTestEntry` schema、确定性 `entryKey` 和规范化 `sourceFingerprint`，并为字段、排序与序列化增加 schema/example 验证。
- [ ] 3.2 实现 Rust 静态候选与 Cargo runtime list adapter，规范化 `target`、`selector`、`sourceRange`，并覆盖 crate target、忽略测试、宏与重复身份诊断。
- [ ] 3.3 实现 Bun 静态候选与 runner report adapter，规范化 suite/test `selector`、`sourceRange`，并覆盖别名、参数化、动态注册与 supporting surfaces。
- [ ] 3.4 实现 smoke task 静态声明与 leaf task report adapter，区分可独立选择的 leaf、聚合 root、helper 和工程校验。
- [ ] 3.5 实现 static/runtime 双向集合核对和稳定机器诊断，分别覆盖 `static-only`、`runtime-only`、`duplicate-entry` 与 `unsupported-entry-shape`，且诊断包含可定位字段。
- [ ] 3.6 从完整当前树生成确定性 machine case inventory，并实现 `missing-case`、`orphan-case`、陈旧 source revision 与重复 case 检查；生成结果不得反向创建源码入口。
- [ ] 3.7 将 `audit.md` 确认的全部 v7 范围外入口纳入同一发现宇宙，并对建 change 时的 81 个 Rust test 与 3 个 Bun supporting test 基线逐项对账；证明每个 supported runtime entry 都有且只有一个静态声明和 machine case。
- [ ] 3.8 为 runner 不可用、list/report 失败、静态扫描失败、inventory 失败和 Claim 失败保留不同错误来源、退出状态与可机器读取结果。

## 4. Evidence Claim、索引与审查查询

- [ ] 4.1 定义 Evidence Claim schema 与受控 topic 表，要求稳定 claim ID、精确 `ownerRef`、非模板 `statement`、可观察 `observations` 和至少一个当前 `supportedBy` `entryKey`。
- [ ] 4.2 实现 Claim 严格校验，覆盖未知 owner、未知 entryKey、空证据集、未知 topic、非法布局、模板复述和 `claim-stale`，同时允许合法 machine case 没有 Claim。
- [ ] 4.3 从当前入口、topic 与 Claims 生成可删除重建的统一 query index 和 case/claim 反向关联，并用 source revision 检出陈旧派生制品。
- [ ] 4.4 实现按 `entryKey`、`runner`、`target`、`sourcePath`、claim ID、精确 topic、`ownerRef` 与文本的有界 list/show 查询，覆盖 case 无 Claim、Claim 多 case 和 case 多 Claim。
- [ ] 4.5 实现索引缺失或陈旧时带 warning 的只读内存投影，证明查询不会隐式写回 inventory、index 或 Claim。
- [ ] 4.6 实现相对明确基线的新增、删除、rename candidate、`implementation-changed` 与 `claim-stale` 报告，并证明该报告只缩小 AI 审查范围、不替代全树闭合检查。
- [ ] 4.7 在 v8 skill 中落实信息增量门槛、owner/观察信号/可靠性审查和 split/merge/rename 流程，明确禁止从测试名、ast-grep 结果或实现断言自动生成 Claim 语义。

## 5. v7 数据迁移与单轨切换

- [ ] 5.1 为 `audit.md` 确认的全部旧 case（建 change 时基线为 431 个）生成 `migration-map.json`，记录旧 case ID 与 topic，并为每项保存唯一迁移终态：目标 `entryKey`、可选目标 Claim 与审查状态，或者没有迁移目标时的终止原因。
- [ ] 5.2 将可机械恢复的 Entry 事实与当前静态/runtime 结果核对后写入 machine inventory；不得把旧 locator 或 marker 当作入口存在性的权威源。
- [ ] 5.3 重新验证 `audit.md` 的模板分类，并与建 change 时 426 个通用 Contract 模板和 396 个 Contract/Proves 双模板基线对账；把无信息增量项写入 `migration-map.json` 的终止原因，不得通过换词把模板保留为 Claim。
- [ ] 5.4 审查 `adapter-contracts` 与 `diagnostics` 的全部非模板候选，逐项核对 owner requirement、可观察信号和证据可靠性，并在 `migration-map.json` 记录目标 Claim 或终止原因。
- [ ] 5.5 审查 `core-cli` 的全部非模板候选，逐项核对 owner requirement、可观察信号和证据可靠性，并在 `migration-map.json` 记录目标 Claim 或终止原因。
- [ ] 5.6 审查 `markdown-adapter` 的全部非模板候选，逐项核对 owner requirement、可观察信号和证据可靠性，并在 `migration-map.json` 记录目标 Claim 或终止原因。
- [ ] 5.7 审查 `navigation` 的全部非模板候选，逐项核对 owner requirement、可观察信号和证据可靠性，并在 `migration-map.json` 记录目标 Claim 或终止原因。
- [ ] 5.8 审查 `output-rendering` 的全部非模板候选，逐项核对 owner requirement、可观察信号和证据可靠性，并在 `migration-map.json` 记录目标 Claim 或终止原因。
- [ ] 5.9 审查 `protocol` 的全部非模板候选，逐项核对 owner requirement、可观察信号和证据可靠性，并在 `migration-map.json` 记录目标 Claim 或终止原因。
- [ ] 5.10 审查 `quality-tooling` 的全部非模板候选，逐项核对 owner requirement、可观察信号和证据可靠性，并在 `migration-map.json` 记录目标 Claim 或终止原因。
- [ ] 5.11 审查 `release` 的全部非模板候选，逐项核对 owner requirement、可观察信号和证据可靠性，并在 `migration-map.json` 记录目标 Claim 或终止原因。
- [ ] 5.12 审查 `shared-foundations` 的全部非模板候选，逐项核对 owner requirement、可观察信号和证据可靠性，并在 `migration-map.json` 记录目标 Claim 或终止原因。
- [ ] 5.13 审查 `test-infrastructure` 的全部非模板候选，逐项核对 owner requirement、可观察信号和证据可靠性，并在 `migration-map.json` 记录目标 Claim 或终止原因。
- [ ] 5.14 校验 `migration-map.json` 与 `audit.md` 的旧 case 集合完全相等，且每项恰好拥有一个完整终态：有目标 `entryKey` 及可选目标 Claim，或有明确终止原因；不得留下重复、缺失或未审查候选。
- [ ] 5.15 更新 `validate:docs -- cases` 和 workspace verifier，使其只从仓库内项目 wrapper 执行全树发现、闭合核对、Claim 与派生制品严格检查。
- [ ] 5.16 同步 `docs/navigation.md`、`docs/testing.md`、`docs/testing/case-maintenance.md`、覆盖材料和 AGENTS 的 owner、读取时机、状态语义与验证命令。
- [ ] 5.17 同步所有仍依赖 v7 语义的 active changes，保留 archived changes 历史原文，并明确 `add-ast-grep-code-adapter` 的产品 crate 与本 change 开发 CLI 互不继承责任。
- [ ] 5.18 在同一个可回滚 changeset 中切换 v8 skill、wrapper、inventory、Claims、query/index、文档和验证入口，随后删除审计固定的全部 v7 逐 case Markdown、源码 marker 与当前验证入口；不得保留双读或第二权威源。
- [ ] 5.19 用 `migration-map.json` 反向演练完整 v7 恢复，证明可以同时恢复 skill、case/topic/index、validator 和文档并移除 v8 制品，而不是只恢复数据或只切 validator。

## 6. 自动化证明与交付验证

- [ ] 6.1 运行全部 ast-grep rule tests，并以 mutation fixtures 证明每条入口规则的正例、近似反例和 unsupported 形态会产生预期稳定结果。
- [ ] 6.2 运行三个 discovery adapter 的单元与集成测试，并为 `missing-case`、`orphan-case`、`duplicate-entry`、`static-only`、`runtime-only`、`unsupported-entry-shape`、陈旧 revision 和 `claim-stale` 各保留失败证明。
- [ ] 6.3 对真实 Cargo、Bun 与 smoke profile 运行静态/runtime/inventory 集合等价检查，证明全部 `entryKey` 可独立报告或选择，且 `audit.md` 确认的全部原遗漏入口已纳入并与建 change 时 84 个漏项基线完成对账。
- [ ] 6.4 运行 Claim schema、topic、query/index、反向关联、只读投影、变更报告和迁移映射测试，证明普通 case 可无 Claim 且每个 Claim 至少有一个当前 case。
- [ ] 6.5 运行 canonical release 构建与 file-set 检查，证明产品包、Docnav CLI 和 `docnav-code` 运行时没有新增 external ast-grep executable、规则或协议行为。
- [ ] 6.6 运行受影响 Rust/Bun/smoke 目标测试、格式化、typecheck、lint、docs validation、严格 OpenSpec validation 和 `bun run verify:docnav-workspace`，把命令、结果与环境限制写入 `verification.md`。
- [ ] 6.7 对稳定文档、AGENTS、代码、skill 与 active changes 做最终搜索，确认除 archived history 和迁移记录外不再依赖 v7 case、marker、旧 template 或双读路径；用局部 diff 核对只修改 change 声明的范围。
