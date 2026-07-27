本 change 准备建立可执行的原生测试入口完整性门禁，并把机器可恢复的 case 事实与需要 AI 判断的 Evidence Claim 分层；本 design 拥有已经选定的实施责任边界和技术决策，但不表示方案已完成阻塞审计或已经实施。

## Context

本节数量是 2026-07-27 建 change 时的观测，只解释决策依据；实施必须在 `audit.md` 重新测量并记录差异，数量变化不自动改变本 design 的责任边界。

当前 `test-evidence-management` 以最小原生测试入口为粒度，但权威源是 431 个手写 case Markdown。仓库内 v7 `check` 校验 topic、case 格式、case ID 和派生索引，不推断 `Entry` 类型、不检查 locator 存在，也不执行或发现测试入口。因此“一入口一 case”只由 agent 流程维持，新增、删除、重命名、拆分、合并或其它 change 合入都可能留下漏登或悬空记录。

v7 迁移只恢复了旧 marker 覆盖范围：310 个 Rust test、104 个 Bun test 和 17 个 smoke leaf task。迁移审计明确排除了 81 个 Rust test 和 3 个 Bun supporting test；当前规则中的“本次范围内”不能证明全仓完整性。现有 431 个 case 中，426 个使用通用 Contract 模板，396 个同时使用通用 Contract 与 Proves 模板，证明逐入口强制写叙述会制造低信息密度状态。

Docnav 同时存在三种稳定差异：

1. Rust 测试由源码 `#[test]` 结构与 Cargo runner 报告共同确定。
2. Bun 测试由 `test` / `it` 注册结构与 Bun runner 报告共同确定。
3. Core smoke 由项目 task factory 最终展开的 leaf task 决定，task 内命令和断言不是独立入口。

`ast-grep` 可以确定性发现 Rust 和 TypeScript 语法形状、输出源码范围并维护正反例规则测试，但不能证明宏展开、类型/引用关系、动态注册或运行时 task 组合。活动 change `add-ast-grep-code-adapter` 使用进程内 Rust crates 实现产品 code adapter；本 change 的开发期结构扫描不得进入该 adapter、Docnav executable、protocol 或 release 制品。

## 使用契约与术语

本节是本 design 的阅读入口，只解释 artifact 权威关系、change 状态和全文术语；具体目标行为由 delta spec 定义，具体实现选择由后续 Decisions 定义。

### Artifact 权威关系

- `proposal.md` 拥有变更动机、能力范围、breaking 范围和影响面。
- 本 `design.md` 拥有已选责任边界、实现策略、取舍、迁移顺序和回滚单位。阻塞审计只能验证可行性和固定实现输入；若审计推翻已选边界，必须先修订并重新校验 change，不得在实现中静默改写。
- `specs/test-evidence-management/spec.md` 拥有归档后应成立的目标行为；change 归档前，它不证明当前仓库已经满足这些要求。
- `tasks.md` 拥有实施依赖顺序和完成状态。实施时产生的 `audit.md`、`migration-map.json` 与 `verification.md` 只保存本 change 的观测和交付证据，不得成为长期测试事实或契约的第二权威源。

### Change 状态

1. **计划与阻塞审计**：v7 case 目录和验证链仍是当前行为；所有 v8 输出都只是审计材料。
2. **Shadow 实现**：新发现器、inventory 和 Claim 检查可以并行运行，但仍不拥有 required 验证或当前事实。
3. **原子切换**：只有阻塞审计通过且新链路满足验收时，v8 才在同一个可回滚 changeset 中成为 required 验证，并同时移除 v7 活跃入口。
4. **归档**：实现与验收完成后再合并 delta spec；归档记录已经完成的切换，不代替实现或验收。

### 术语

- **supported runner profile**：仓库内版本化的覆盖边界，显式列出纳入 required 门禁的 Cargo test targets、Bun test surfaces、smoke task roots 及其确定性 list/report 参数。
- **完整当前树**：调用检查时，当前工作树中属于 supported runner profile 的全部源码声明和 runner 报告；不得按 Git diff、当前 change、旧 marker 或人工抽样缩小。
- **`NativeTestEntry`**：同一个原生测试入口的静态声明与 runner 身份成功归一后形成的共享记录。
- **machine case**：从一个 `NativeTestEntry` 确定性生成的入口事实投影；machine case 不拥有手写语义。
- **Evidence Claim**：由稳定 owner requirement、不可从测试名机械恢复的判断、可观察结果和一个或多个当前 machine case 组成的长期语义记录；machine case 可以没有 Claim。
- **离线 required 验证**：required check 本身不访问网络、不运行 updater，也不依赖个人 PATH。依赖获取由仓库声明的固定工具链和 lockfile 承接，其具体 bootstrap 条件必须在阻塞审计中固定。

## Goals / Non-Goals

**Goals:**

- 让当前树中所有受支持 runner 的最小原生测试入口都能被确定性发现，并恰好形成一个 machine case。
- 用静态结构结果、runner 报告与 committed inventory 的集合相等检查阻止漏登、悬空、重复和静默排除。
- 将 case 限定为机器入口事实，将长期契约和可观察证明迁移为可关联多个 case 的 Evidence Claim。
- 对新增、删除、重命名候选、实现变化、owner 变化和不支持形态提供稳定机器诊断，供 AI 做有界审查。
- 保持 required 验证离线、仓库内、可复现，并保留有界查询与可删除重建的派生索引。
- 以单轨迁移替换 v7 逐 case Markdown，不保留 marker、双读或两套语义 owner。

**Non-Goals:**

- 不自动判断产品实现变化后测试是否充分，也不把 AST 匹配、runner 通过或 source fingerprint 当作 Evidence Claim 的语义证明。
- 不自动从测试名称、断言或 owner 文档生成 Contract/Proves 叙述。
- 不改变 Docnav 产品 CLI、adapter、protocol、ref、输出、配置或 canonical release。
- 不让开发期 ast-grep executable 复用或替代 `docnav-code` 的进程内 ast-grep Rust crates。
- 不改写已归档 OpenSpec change 的历史术语和迁移证据。

## Decisions

### Decision 1: 全部受支持原生入口进入同一个覆盖宇宙

项目定义一个版本化的 supported runner profile，列出纳入 required 门禁的 Cargo test targets、Bun test surfaces 和 smoke task roots。完整性检查每次读取最终当前树，不以“本次触及”“旧 marker 覆盖”或 Git diff 作为纳入条件。lint、类型检查、schema、生成物一致性、CI job、fixture、helper、hook、断言和测试步骤仍不形成 machine case。

影响：阻塞审计确认的全部 v7 范围外入口（建 change 时基线为 81 个 Rust test 和 3 个 Bun supporting test）必须在切换前逐项进入 inventory；它们可以没有 Evidence Claim，但不能从入口集合中消失。

### Decision 2: 共享 NativeTestEntry 核心，runner 差异留在发现 adapter

三个发现 adapter 输出同一个规范化 `NativeTestEntry`：

- `entryKey`：由 runner、target 和 runner selector 组成的确定性当前身份。
- `runner` 与 `target`：区分 Cargo test binary、Bun test surface 和 smoke root。
- `selector`：runner 能稳定单独报告或选择的名称。
- `sourcePath` 与 `sourceRange`：静态声明位置。
- `sourceFingerprint`：从规范化入口 AST 或稳定 task declaration 计算的内容摘要。

Rust adapter、Bun adapter 和 smoke adapter 分别拥有静态/运行时获取方式、名称规范化与不支持形态诊断。共享层不理解 `#[test]`、`it`、Cargo target、Bun suite 或 smoke object shape。

替代方案是用一个跨语言 scanner 直接猜测全部入口。该方案会把宏、suite、动态注册和 task 展开差异隐藏在共享分支中，因此不采用。

### Decision 3: ast-grep 只拥有静态结构候选和规则回归

项目首次接入固定来源的完整 `.codex/skills/ast-grep/` 分发，并为 Rust、Bun、smoke candidate 和不支持动态形态建立 project rules 与 rule tests。每条规则必须有应匹配正例和最接近但不应匹配的反例；机器消费使用 JSON stream，并保留 path、range 和捕获字段。

实现前审计必须固定 skill source commit/release、开发期 ast-grep CLI 的安装来源与精确版本，并证明依赖由仓库声明的固定 bootstrap 准备后，required check 可以通过项目工具入口运行且不访问网络。2026-07-27 建 change 时的环境观测未找到 `ast-grep` / `sg`；审计必须重新检查，并且不得把隐式个人安装当作前置。

开发期 CLI、rules 和 snapshots 只属于测试验证工具链。`add-ast-grep-code-adapter` 继续使用它自己精确锁定的进程内 Rust crates；release package file set 和产品运行时不得新增外部 ast-grep executable。

### Decision 4: 静态、运行时和 inventory 必须闭合

required check 计算并比较：

```text
static entries <-> runtime entries <-> machine case inventory
```

正常入口必须同时具有可规范化的静态声明和 runner 身份。检查至少报告：

- `missing-case`：静态/运行时入口没有 inventory case。
- `orphan-case`：inventory case 没有当前入口。
- `duplicate-entry`：多个 case 或声明归一为同一入口。
- `static-only`：源码候选未进入 runner。
- `runtime-only`：runner 入口无法绑定静态声明。
- `unsupported-entry-shape`：动态生成、alias、wrapper、宏或 task 组合没有受支持 adapter。

不支持形态不得被自动排除。维护者必须扩展并测试 adapter，或把测试重构为已有稳定形态后才能通过。参数化测试按 runner 的稳定独立报告粒度处理；无法稳定枚举时整个形态阻断。

### Decision 5: Machine case 是生成事实，不是手写证据文章

每个 `NativeTestEntry` 恰好生成一个 machine case；case ID 使用当前确定性 `entryKey`，不承诺测试重命名后保持身份。长期连续性由 Claim ID 承接，测试重命名表现为 orphan/missing 候选并由 AI 重新关联。

Machine case 不包含手写 `Contract`、`Proves`、Status、角色或 Verification，也不使用源码 marker。Committed inventory 是便于离线查询和 Git 审计的派生制品，可以从当前发现结果删除重建，不能反向创建或补造测试入口。

替代方案是保留 431 个 Markdown，只自动验证 locator。该方案仍要求为每个入口维护重复文件和模板字段，不能减少主要维护面，因此不采用。

### Decision 6: Evidence Claim 单独拥有长期语义

Evidence Claim 使用稳定 claim ID，并至少包含：

- 精确 `ownerRef`：定位稳定 owner requirement，而不是笼统文件名。
- `statement`：不能从测试名称机械恢复的契约判断。
- `observations`：调用方可观察的输出、错误、状态、交互或资源结果。
- `supportedBy`：一个或多个当前 machine case `entryKey`。

Claim 与 case 是多对多关系；普通内部入口允许零个 Claim。Topic 只组织 Claim 的稳定责任，不再给每个 machine case 强制分配人工 topic。一个 Claim 没有当前 case、引用未知 owner、含未知 entryKey 或只使用已知模板复述时严格检查失败。

证据审查继续检查契约背景、失败信号、可观察性、可靠性、证据独立性和维护价值。ast-grep 结果、测试名称和实现断言不得自动生成 Claim 语义。

### Decision 7: 事实源、派生索引和查询按责任分层

事实源固定为：

1. 当前源码与 runner 报告拥有原生入口存在性和身份。
2. Claim Markdown 拥有 `ownerRef`、`statement`、`observations` 和 `supportedBy`。
3. 受控 topic 表拥有 Claim topic。
4. Machine case inventory、统一 query index 和反向 claim/case 关联都是可删除重建的派生制品。

查询层必须支持按 `entryKey`、`runner`、`target`、`sourcePath`、claim ID、topic、`ownerRef` 和文本有界过滤，并能展开单个 Claim 及其当前 cases、单个 case 及其 Claims。索引缺失或陈旧时可以构造带 warning 的只读内存投影，不得隐式写回。

### Decision 8: 结构变化由机器分类，语义变化由 AI 审查

项目提供相对基线的变更报告：

- 新增、删除和 selector/path 变化来自入口集合差异。
- 同一 entryKey 的 `sourceFingerprint` 变化标记为 `implementation-changed`。
- Claim ownerRef 指向内容或 linked case 集合变化时标记为 `claim-stale`。

Machine inventory 的当前树检查不依赖基线；基线只用于给 AI 缩小审查范围。AI 对 rename、split、merge、Claim 继续成立或 Claim 修改作出判断，并在 change 级迁移/审查记录中保存结论，不为每个未改变 case 写重复审核模板。

只修改产品实现而没有改变测试入口、测试体或 owner 文档时，集合与 fingerprint 不能证明测试充分性。该情况继续由项目变更流程结合 owner 文档、CodeGraph 影响面和目标测试审查，不能被本门禁宣称为已证明。

### Decision 9: 项目 wrapper 拥有发现，通用 skill 拥有审查模型

仓库内项目 wrapper 负责运行 ast-grep、Cargo/Bun/smoke 入口、规范化 runner adapter、集合比较和当前树 inventory 生成。`test-evidence-review` skill v8 负责机器结果分类、Claim 质量、AI 审查流程、通用 schema/query/index 契约和完成标准。

`validate:docs -- cases` 调用项目 wrapper；wrapper 可以导入 skill 的通用模块，但不得复制 Claim/catalog 规则。发现失败、测试执行失败、inventory 失败和 Claim 失败必须保持不同诊断来源。

### Decision 10: v7 单轨迁移且保留完整去向

迁移先生成全树入口基线和静态/运行时差异，再从阻塞审计确认的全部旧 case 抽取：

- Entry 事实进入 machine inventory。
- 非模板 Contract/Proves 只作为 Claim 候选，必须重新核对 owner 和证明信号。
- 模板内容不改写成新措辞；没有信息增量时不建立 Claim。
- 旧 case ID、目标 entryKey、目标 claim ID 或终止原因进入一次性迁移映射。

实施观测固定写入 change-local `audit.md`，逐 case 去向固定写入 `migration-map.json`，实际运行的验收命令与结果固定写入 `verification.md`。这些文件只支持本 change 审计和回滚，不进入长期测试证据查询。

切换提交同时更新 skill、发现 rules、inventory、Claims、query/index、validator、文档和 active changes，并删除旧逐 case Markdown 语义。不得提供 v7/v8 双读或把归档 change 当作当前来源。

## Risks / Trade-offs

- [Risk] ast-grep 规则漏掉 alias、宏或动态注册。→ 静态与 runner 结果双向比较，runtime-only 和 unsupported 形态阻断；每条规则维护正反例。
- [Risk] runner 列表受 feature、target 或环境影响。→ supported runner profile 固定命令、features、环境和 target 身份；实现审计记录代表性差异并加入 fixture。
- [Risk] source fingerprint 因格式化产生噪声。→ 从规范化 AST/task declaration 计算，先测量当前树稳定性；无法稳定规范化的 runner 使用显式 adapter 策略。
- [Risk] 允许 inventory-only case 会弱化“每个测试都有契约说明”的表面覆盖。→ 完整性与语义价值分开报告；Claim 覆盖率只是审计信号，不伪装为测试充分性。
- [Risk] AI 可以机械刷新派生 revision 而不完成语义审查。→ required gate 只宣称结构和 stale 状态；高风险 Claim 的变更结论保存在 change 级审查记录并由 code review/验收检查。
- [Risk] 全树扫描和 runner 交叉核对增加 required 时间。→ 复用现有 Cargo/Bun/smoke 执行材料，测量冷/热基线；只有不削弱完整性时才使用缓存。
- [Risk] 开发期 ast-grep 与 code adapter 的 ast-grep 依赖被误合并。→ 分别记录 owner、进程边界和 release file-set test；跨 change 合并只协调 lockfile/工具配置，不共享产品实现。
- [Risk] 删除 v7 case Markdown 造成审计信息丢失。→ 实现前固定审计集合、模板/非模板分类、完整迁移映射和可恢复备份；归档 v7 change 保持原文。

## Migration Plan

1. 完成阻塞级审计：固定 proposal/design/spec/tasks 一致性、上游 skill/CLI 来源、当前 runner universe、全部 v7 case 分类、全部未纳入入口、active change 冲突和回滚单位，并与建 change 时的 431/84 数量基线对账。
2. 接入固定 ast-grep skill 与开发工具，建立 Rust/Bun/smoke 正反例规则和 unsupported-shape 规则测试。
3. 实现三个 runner adapter、统一 `NativeTestEntry`、静态/runtime 双向比较、全树 inventory 与稳定机器诊断；先并行生成审计结果，不切换 v7 owner。
4. 实现 Evidence Claim schema、topic、查询/反向关联、stale 检测和通用 skill v8 工作流。
5. 逐项迁移审计确认的全部旧 case 并补齐此前未纳入的原生入口；运行真实 runner 验证全部 entryKey 可达，审计 Claim 候选而不自动生成叙述。
6. 原子切换 `validate:docs -- cases`、workspace verifier、稳定文档、AGENTS 和相关 active changes，删除旧逐 case Markdown 与 v7 单轨入口。
7. 运行 rule tests、发现 adapter tests、catalog tests、目标 Rust/Bun/smoke tests、docs validation、typecheck/lint、严格 OpenSpec 和 full workspace verification。
8. 回滚时作为一个 changeset 恢复审计固定的 v7 skill、case Markdown、topic/index、validator 和文档，删除 v8 skill、ast-grep developer integration、discovery rules、machine inventory 与 Claims；不得只恢复数据或只切换 validator。

## Open Questions

无未回答开放问题，可以进入实现前阻塞审计。ast-grep skill source、CLI 精确版本和 runner 命令矩阵属于审计必须固定的实现输入，不改变本 design 的责任边界。
