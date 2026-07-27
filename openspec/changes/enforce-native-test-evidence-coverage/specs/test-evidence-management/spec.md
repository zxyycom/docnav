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
- TO: `### Requirement: 仓库内验证只使用当前测试证据链`

## MODIFIED Requirements

### Requirement: 每个受支持的最小原生测试入口对应一个 machine case
项目 MUST 维护版本化 supported runner profile，显式列出纳入 required 门禁的
Cargo test targets、Bun test surfaces、smoke task roots 及其确定性 list/report
参数。测试证据 MUST 以 runner 能稳定独立选择或报告且拥有完整测试意图的最小原生
入口为粒度。完整当前树中的每个闭合入口 MUST 恰好生成一个 machine case；聚合
容器、内部环节和工程校验 MUST NOT 生成 case。

#### Scenario: Runner 报告多个原生节点
- **WHEN** 一个文件、suite、脚本或 CI job 聚合多个可独立报告的测试节点
- **THEN** 聚合对象只作为执行或定位容器
- **THEN** 每个最小原生入口分别生成一个 machine case

#### Scenario: Helper 只服务所属入口
- **WHEN** fixture、helper、mock、hook、断言或步骤只参与一个测试节点的最终判定
- **THEN** 它只影响所属入口的实现与 fingerprint
- **THEN** 它不得生成独立 machine case

#### Scenario: 完整当前树出现新入口
- **WHEN** 任一分支、合并或其它 change 使 supported runner profile 出现新入口
- **THEN** strict check 报告 `missing-case` 并失败
- **THEN** 该入口不得因不在当前 Git diff 或人工审查范围而被忽略

#### Scenario: Supported runner profile 改变
- **WHEN** 维护者修改 runner target、test surface、smoke root 或 list/report 参数
- **THEN** strict check 按新 profile 重算完整 static、runtime 与 inventory 集合
- **THEN** profile 变更必须有 owner-backed 原因

#### Scenario: 工程校验不是测试入口
- **WHEN** lint、类型检查、schema、生成物一致性或安全扫描只执行工程校验
- **THEN** 该校验由自身 owner 和验证链路承接
- **THEN** machine inventory 不把命令、job 或结果登记为测试 case

### Requirement: Machine case inventory 与 Evidence Claim 分层
项目 MUST 让当前源码与 runner 报告拥有入口事实，让 Claim Markdown 拥有长期 owner
语义，并从入口事实生成 machine inventory。一个 Claim MUST 关联一个或多个 case；
项目 MUST 允许一个 case 关联零个或多个 Claim。Topic MUST 只组织当前 Claim 的
稳定责任，不得改变 case 身份或粒度。

#### Scenario: 普通内部测试没有长期 Claim
- **WHEN** 一个合法入口只有局部实现验证价值
- **THEN** 它仍恰好出现在 machine inventory
- **THEN** 系统不得为字段完整性生成 Contract、Proves 或 Evidence Claim

#### Scenario: 一个判断由多个入口证明
- **WHEN** 多个入口共同观察同一 owner requirement 的不同代表或边界
- **THEN** 一个 Evidence Claim 可以关联这些 machine cases
- **THEN** 每个 machine case 仍只代表自己的原生入口

#### Scenario: Claim topic 无当前消费者
- **WHEN** 一个 topic 没有任何当前 Claim 使用
- **THEN** 维护者从受控 topic 表删除该分类
- **THEN** 系统不得为预留分类生成空 Claim

### Requirement: Machine case 只表达可发现入口事实
每个 machine case MUST 使用确定性 `entryKey` 作为当前身份，并只投影 `runner`、
`target`、`selector`、`sourcePath`、`sourceRange` 和 `sourceFingerprint` 等可从
发现结果恢复的事实。Machine case MUST NOT 拥有手写 Contract、Proves、Status、
角色或 Verification，也 MUST NOT 依赖源码 marker。

#### Scenario: 静态声明与 runner 身份闭合
- **WHEN** 一个静态声明和一个 runner 节点可归一为同一入口
- **THEN** inventory 生成一个包含稳定事实的 machine case
- **THEN** 相同 entryKey 的第二条声明或 case 被报告为 `duplicate-entry`

#### Scenario: 测试重命名但语义继续
- **WHEN** runner selector 改变，使当前 baseline 的旧 entryKey 消失并出现新 entryKey
- **THEN** 变化报告提供 rename candidate
- **THEN** AI 判断 Claim 是否重新关联，不由 inventory 猜测语义连续性

#### Scenario: 实现变化但入口身份不变
- **WHEN** `entryKey` 不变但规范化 `sourceFingerprint` 改变
- **THEN** 当前 inventory 更新机器事实
- **THEN** 变化报告标记 `implementation-changed` 供 AI 审查

### Requirement: 派生索引可重建且查询有界
项目 MUST 从当前 inventory、受控 topic 和合法 Claims 生成统一 query index。派生
制品 MUST 可删除重建，并提供按 `entryKey`、runner、target、sourcePath、Claim ID、
精确 topic、`ownerRef` 和文本的有界查询；派生制品不得成为入口、Claim 或 topic 的
第二权威源。

#### Scenario: 当前来源变化使索引陈旧
- **WHEN** inventory、topic、Claim 正文、owner section 或 `supportedBy` 改变
- **THEN** strict check 识别 stale index
- **THEN** 同步入口可从全部合法当前来源重建派生制品

#### Scenario: 查询使用内存投影
- **WHEN** committed index 缺失或陈旧但当前来源合法
- **THEN** `list` / `show` 可以返回带 warning 的只读内存投影
- **THEN** 查询不得隐式写回 inventory、index 或 Claim

#### Scenario: 从 case 反查 Claim
- **WHEN** 调用方展开一个当前 machine case
- **THEN** 查询返回入口事实和全部关联 Claim
- **THEN** 没有关联 Claim 的合法 case 返回空 Claim 集合

### Requirement: Evidence Claim 保持可观察与可靠
每个 Evidence Claim MUST 使用按稳定语义命名的全局唯一 ID，精确引用当前 owner
requirement，表达不能从测试名称机械恢复的 statement，并描述调用方可判断的
observations。Claim 所关联测试 MUST 检查 fixture、mock、时序、随机性和环境不会使
证据失真。Claim MUST NOT 复述实现、只证明 mock、让被测实现生成自身预期值或使用
通用模板填充语义字段。

#### Scenario: 测试证明公共失败语义
- **WHEN** 一个或多个入口验证 owner 定义的错误或失败边界
- **THEN** Claim 的 ownerRef 精确定位该稳定边界
- **THEN** observations 说明调用方可观察的错误、状态、交互或资源结果

#### Scenario: 内容可从测试名恢复
- **WHEN** 候选 Claim 只复述入口名称已经表达的结果
- **THEN** AI 审查拒绝把该文字保存为长期 Claim
- **THEN** 对应入口只保留 machine case

#### Scenario: 测试混合独立意图
- **WHEN** 一个测试节点包含可独立命名和失败的多个意图
- **THEN** 维护者先拆分原生测试节点
- **THEN** 每个保留节点分别进入 inventory，Claim 再按稳定判断关联

#### Scenario: Claim 失去当前证据
- **WHEN** owner requirement 不存在、supportedBy 含未知 entryKey 或全部入口被删除
- **THEN** strict check 报告 `claim-stale` 或对应阻断诊断
- **THEN** Claim 必须修订、重新关联或删除

### Requirement: 仓库内验证只使用当前测试证据链
项目 MUST 通过仓库内项目 wrapper 对 supported runner profile、静态规则、runner
报告、machine inventory、Claims 和 query index 执行确定性严格检查。Required
验证 MUST NOT 依赖个人 skill、浮动网络资源、源码 marker、旧账本或 updater。

#### Scenario: Required profile 检查完整当前树
- **WHEN** 本地或 CI 运行 required 文档验证
- **THEN** `cases` task 执行 static/runtime/inventory 集合核对和 Claim 严格检查
- **THEN** 漏项、悬空、重复、不支持形态或 stale Claim 使任务失败

#### Scenario: 开发期 ast-grep 与产品隔离
- **WHEN** 项目运行 ast-grep CLI 发现测试入口
- **THEN** executable 和 rules 只属于开发验证依赖
- **THEN** canonical release 不包含或运行 external ast-grep

#### Scenario: 测试变更维护证据
- **WHEN** 任务新增、修改、删除、重命名、拆分或合并测试
- **THEN** strict check 先证明完整当前树的一入口一 machine case
- **THEN** AI 只对变化入口和受影响 Claim 做语义审查，并运行目标测试

## ADDED Requirements

### Requirement: 静态结构与 runner 报告必须双向核对
每个 runner adapter MUST 把静态声明和实际 runner 报告规范化为同一
NativeTestEntry 模型，并双向报告不能匹配的成员。Ast-grep 规则 MUST 有正例和最接近
反例；动态注册、宏、alias、wrapper、参数化或 task 组合无法可靠归一时 MUST 产生
`unsupported-entry-shape`。

#### Scenario: 静态入口未进入 runner
- **WHEN** 项目规则发现声明但固定 runner profile 不报告该入口
- **THEN** strict check 报告 `static-only` 并失败
- **THEN** 诊断包含可用的 runner、target、source path/range 和 selector

#### Scenario: Runner 入口没有静态声明
- **WHEN** runner 报告入口但项目规则不能绑定声明
- **THEN** strict check 报告 `runtime-only` 或 `unsupported-entry-shape`
- **THEN** 维护者扩展 adapter 或把入口收敛为受支持形态

#### Scenario: 近似结构不是入口
- **WHEN** helper、fixture、普通函数、suite 或聚合器与入口语法相近
- **THEN** rule test 的反例证明这些结构不被登记
- **THEN** runtime/inventory 不为其生成 machine case

### Requirement: 变化报告只缩小审查范围而不定义完整性
项目 MUST 能相对显式 baseline 报告新增、删除、rename candidate、
implementation-changed 和 claim-stale，但完整性 MUST 始终由当前树全量核对决定。
产品实现变化未改变入口、测试源码或 owner 时，系统 MUST NOT 声称测试充分性已经由
inventory 证明。

#### Scenario: 其它 change 合入测试
- **WHEN** 当前分支合入另一 change 新增的入口
- **THEN** 全树 strict check 仍报告对应 missing case 或 stale artifact
- **THEN** 变化来源不改变修复义务

#### Scenario: 只修改产品实现
- **WHEN** 产品代码变化但入口、测试 fingerprint 和 Claim owner 均未变化
- **THEN** machine inventory 可以保持结构合法
- **THEN** 交付流程仍按 owner、影响面和目标测试判断证据是否充分
