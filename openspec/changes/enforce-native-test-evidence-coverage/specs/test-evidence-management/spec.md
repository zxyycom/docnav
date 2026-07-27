本 delta 准备把 `test-evidence-management` 从人工逐 case 文档升级为全树原生入口完整性门禁和独立 Evidence Claim；它定义目标行为，不证明 change 已审核或已经实施。

## RENAMED Requirements

- FROM: `### Requirement: 一个保留的最小原生测试入口对应一个 case`
- TO: `### Requirement: 每个受支持的最小原生测试入口对应一个 machine case`
- FROM: `### Requirement: Topic 目录与单 case Markdown 是权威源`
- TO: `### Requirement: Machine case inventory 与 Evidence Claim 分层`
- FROM: `### Requirement: Case 只表达入口、契约和证明`
- TO: `### Requirement: Machine case 只表达可发现入口事实`
- FROM: `### Requirement: 证据评估保持可观察与可靠`
- TO: `### Requirement: Evidence Claim 保持可观察与可靠`
- FROM: `### Requirement: 仓库内验证只使用单一测试证据源`
- TO: `### Requirement: 仓库内验证只使用当前 v8 权威链`

## MODIFIED Requirements

### Requirement: 每个受支持的最小原生测试入口对应一个 machine case
项目 MUST 在仓库内维护版本化的 supported runner profile，显式列出纳入 required 门禁的 Cargo test targets、Bun test surfaces、smoke task roots 及其确定性 list/report 参数。测试证据 MUST 以测试框架能够稳定独立选择、单独报告且拥有完整测试意图的最小原生测试入口为 machine case 粒度。supported runner profile 覆盖的完整当前树中，每个静态与运行时可核对的入口 MUST 恰好生成一个 case；聚合容器、内部环节和工程校验 MUST NOT 生成 case。

#### Scenario: Runner 报告多个原生测试节点
- **WHEN** 一个测试文件、suite、脚本或 CI job 聚合多个可区分的原生测试节点
- **THEN** 聚合对象只作为执行或定位容器
- **THEN** 每个最小原生测试入口分别生成一个 machine case

#### Scenario: Helper 只服务所属测试入口
- **WHEN** fixture、helper、mock、hook、断言或步骤只参与一个原生测试节点的最终判定
- **THEN** 它只影响所属入口的实现与 fingerprint
- **THEN** 它不得生成独立 machine case

#### Scenario: 完整当前树出现未登记入口
- **WHEN** 任一分支、合并或其它 change 使 supported runner profile 出现新的原生测试入口
- **THEN** required check 报告 `missing-case` 并失败
- **THEN** 该入口不得因不在本次 Git diff、旧 marker 范围或人工审查范围而被忽略

#### Scenario: Supported runner profile 发生变化
- **WHEN** 维护者新增、修改或删除一个 runner target、test surface、smoke root 或 list/report 参数
- **THEN** required check 按新 profile 重新计算完整静态、runtime 与 inventory 集合
- **THEN** profile 变更必须有 owner-backed 原因，不得只为绕过现有完整性诊断而缩小覆盖边界

#### Scenario: 工程校验不是测试入口
- **WHEN** lint、类型检查、schema、生成物一致性、安全扫描或 workspace profile 只执行工程校验
- **THEN** 该校验由自身 owner 和验证链路承接
- **THEN** machine inventory 不把命令、job 或结果登记为测试 case

### Requirement: Machine case inventory 与 Evidence Claim 分层
项目 MUST 让当前源码与 runner 报告拥有原生入口事实，让 Claim Markdown 拥有长期 owner 语义，并 MUST 从入口事实生成 machine case inventory。一个 Claim MUST 关联一个或多个 case；项目 MUST 允许一个 case 关联零个或多个 Claim。Topic MUST 只组织 Claim 的稳定责任，不得改变 machine case 身份或测试粒度。

#### Scenario: 普通内部测试没有长期 Claim
- **WHEN** 一个合法原生入口只有局部实现验证价值且没有需要长期沉淀的 owner-backed 判断
- **THEN** 它仍恰好出现在 machine case inventory
- **THEN** 系统不得为满足字段完整性自动生成 Contract、Proves 或 Evidence Claim

#### Scenario: 一个稳定判断由多个入口证明
- **WHEN** 多个原生测试入口共同观察同一 owner requirement 的不同代表或边界
- **THEN** 一个 Evidence Claim 可以在 `supportedBy` 中关联这些 machine case
- **THEN** 每个 machine case 仍只代表自己的原生测试入口

#### Scenario: Claim topic 未定义
- **WHEN** Claim 位于未知 topic、topic 表非法或 Claim 布局不符合固定目录契约
- **THEN** strict check 返回阻断诊断
- **THEN** 派生 inventory 或 query index 不得掩盖该错误

### Requirement: Machine case 只表达可发现入口事实
每个 machine case MUST 使用确定性 `entryKey` 作为当前身份，并 MUST 只投影 `runner`、`target`、`selector`、`sourcePath`、`sourceRange` 和 `sourceFingerprint` 等可从发现结果恢复的入口事实。Machine case MUST NOT 拥有手写 Contract、Proves、Status、角色或 Verification，也 MUST NOT 依赖源码 marker。

#### Scenario: 静态声明与 runner 身份闭合
- **WHEN** 一个静态测试声明和一个 runner 报告节点可以规范化为同一入口
- **THEN** inventory 生成一个包含两侧稳定事实的 machine case
- **THEN** 相同 entryKey 的第二条声明或 case 被报告为 `duplicate-entry`

#### Scenario: 测试重命名但语义继续
- **WHEN** runner selector 或稳定 source identity 改变，使旧 entryKey 消失并出现新 entryKey
- **THEN** 变更报告提供 orphan 与 missing 的 rename candidate
- **THEN** AI 可以把现有 Claim 重新关联到新 case，而不得由 inventory 猜测长期语义连续性

#### Scenario: 实现变化但入口身份不变
- **WHEN** `entryKey` 保持不变但规范化 `sourceFingerprint` 改变
- **THEN** 当前 inventory 更新机器事实
- **THEN** 变更报告把该入口标记为 `implementation-changed` 供 AI 审查

### Requirement: 派生索引可重建且查询有界
项目 MUST 从完整当前入口发现结果、受控 topic 表和全部合法 Evidence Claim 生成 machine case inventory 与统一 query index。派生制品 MUST 可删除重建，并 MUST 提供按 `entryKey`、`runner`、`target`、`sourcePath`、claim ID、精确 topic、`ownerRef` 和文本的有界查询；派生制品不得成为入口存在性、Claim 内容或 topic 的第二权威源。

#### Scenario: 入口或 Claim 变化使索引陈旧
- **WHEN** 当前入口集合、`sourceFingerprint`、topic、Claim 路径、正文、`ownerRef` 或 `supportedBy` 发生变化
- **THEN** 旧 source revision 被识别为陈旧
- **THEN** strict check 失败，且同步入口可以从全部合法权威源原子重建派生制品

#### Scenario: 查询使用合法内存投影
- **WHEN** committed inventory 或 query index 缺失或陈旧，但入口发现结果、topic 和 Claim 全部合法
- **THEN** 查询可以返回带 warning 的只读内存投影
- **THEN** 查询不得隐式写回 inventory、索引或 Claim

#### Scenario: 从 case 反查 Claim
- **WHEN** 调用方展开一个当前 machine case
- **THEN** 查询返回该入口的机器事实和全部关联 Claim
- **THEN** 没有关联 Claim 的合法 case 返回空 Claim 集合而不是合成叙述

### Requirement: Evidence Claim 保持可观察与可靠
每个 Evidence Claim MUST 使用稳定 claim ID，精确引用当前 owner requirement，表达不能从测试名称机械恢复的 statement，并描述调用方可判断的 observations。Claim 所关联的测试 MUST 检查输入、fixture、mock、时序、随机性和环境不会使证据失真；Claim MUST NOT 只复述实现、只证明 mock、让被测实现生成自身预期值或使用已知模板填充语义字段。

#### Scenario: 测试证明公共失败语义
- **WHEN** 一个或多个原生测试入口验证 owner 定义的错误或失败边界
- **THEN** Claim 的 ownerRef 精确定位该稳定边界
- **THEN** observations 说明调用方可观察的错误、状态、交互或资源结果

#### Scenario: 内容可从测试名机械恢复
- **WHEN** 候选 Claim 只说明“该入口验证其名称所描述的结果”或笼统声明某文档定义相关行为
- **THEN** strict check 或 AI 审查拒绝把该文字作为长期 Evidence Claim
- **THEN** 对应原生入口只保留 machine case，直到存在真实信息增量

#### Scenario: 测试混合独立意图
- **WHEN** 一个原生测试节点包含可以独立命名和独立失败的多个测试意图
- **THEN** 维护者先拆分测试节点
- **THEN** 每个保留节点分别进入 machine inventory，相关 Claim 再按稳定判断关联

#### Scenario: Claim 失去当前证据
- **WHEN** Claim 的 owner requirement 不存在、supportedBy 含未知 entryKey 或全部关联入口被删除
- **THEN** strict check 报告 `claim-stale` 或对应阻断诊断
- **THEN** Claim 必须被修订、重新关联、转交或删除

### Requirement: 仓库内验证只使用当前 v8 权威链
项目 MUST 跟踪固定来源的 test-evidence-review v8 与 ast-grep skill 完整分发，并 MUST 通过仓库内项目 wrapper 对 supported runner profile、静态规则、runner 报告、machine inventory、Claims 和派生索引执行确定性严格检查。Required 验证 MUST NOT 依赖个人 skill、浮动网络资源、源码 marker、v7 逐 case Markdown 或 updater。

#### Scenario: Required profile 检查完整当前树
- **WHEN** 本地或 CI 运行 required 文档验证
- **THEN** `cases` task 从仓库跟踪路径执行静态/runtime/inventory 集合核对和 Claim 严格检查
- **THEN** 漏登、悬空、重复、static-only、runtime-only、不支持形态或陈旧 Claim 使任务失败

#### Scenario: 开发期 ast-grep 与产品制品隔离
- **WHEN** 项目安装或运行 ast-grep CLI 以发现测试入口
- **THEN** 该 executable 和规则只属于开发验证依赖
- **THEN** canonical release 不包含或运行外部 ast-grep，`docnav-code` 仍遵守自己的进程内 Rust crate 边界

#### Scenario: 测试变更维护证据
- **WHEN** 任务新增、修改、删除、重命名、拆分、合并或保留测试实现
- **THEN** required check 先证明完整当前树的一入口一 machine case
- **THEN** AI 只对变化入口和受影响 Claim 完成语义审查，并运行目标测试

#### Scenario: 归档历史保留旧术语
- **WHEN** 已归档 OpenSpec change 记录迁移前的账本、marker 或 v7 逐 case Markdown
- **THEN** 历史 artifact 可以保持原文
- **THEN** 稳定文档、AGENTS、代码和 active changes 不得继续依赖旧流程

## ADDED Requirements

### Requirement: 静态结构与 runner 报告必须双向核对
每个 runner adapter MUST 把静态声明和实际 runner 报告规范化为同一 NativeTestEntry 模型，并 MUST 双向报告不能匹配的成员。Ast-grep 规则 MUST 由正例和最接近反例验证；动态注册、宏、alias、wrapper、参数化或 task 组合无法可靠归一时 MUST 产生 `unsupported-entry-shape`，不得静默排除。

#### Scenario: 静态入口未进入 runner
- **WHEN** ast-grep 或项目静态 adapter 发现测试声明，但固定 runner profile 不报告对应入口
- **THEN** strict check 报告 `static-only` 并失败
- **THEN** 诊断包含 `runner`、`target`、`sourcePath`、`sourceRange` 和可用 `selector`

#### Scenario: Runner 入口没有静态声明
- **WHEN** 固定 runner profile 报告入口，但项目规则不能绑定对应源码声明
- **THEN** strict check 报告 `runtime-only` 或 `unsupported-entry-shape` 并失败
- **THEN** 维护者必须扩展并测试 adapter，或把入口收敛为受支持形态

#### Scenario: 规则近似结构不是测试入口
- **WHEN** helper、fixture、普通函数、suite 容器或 task 聚合器与测试入口具有相近语法
- **THEN** ast-grep rule test 的反例证明这些结构不被登记
- **THEN** runtime/inventory 不为这些结构生成 machine case

### Requirement: 变化报告只缩小审查范围而不定义完整性
项目 MUST 能相对明确基线报告新增、删除、rename candidate、implementation-changed 和 claim-stale 项，但完整性 MUST 始终由当前树全量集合核对决定。产品实现变化未改变入口、测试源码或 owner 时，系统 MUST NOT 声称测试充分性已经由 inventory 证明。

#### Scenario: 其它 change 合入测试
- **WHEN** 当前分支合入另一 change 新增的测试入口，而该入口不在原任务 diff
- **THEN** 全树 required check 仍报告对应 missing case 或生成物陈旧
- **THEN** 变化来源不改变修复义务

#### Scenario: 只修改产品实现
- **WHEN** 产品代码变化但入口集合、测试源码 fingerprint 和 Claim owner 均未变化
- **THEN** machine inventory 可以保持结构合法
- **THEN** 交付流程仍必须通过 owner、影响面和目标测试审查判断证据是否充分
