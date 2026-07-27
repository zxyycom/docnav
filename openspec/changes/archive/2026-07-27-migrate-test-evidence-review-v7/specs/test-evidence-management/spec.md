本 spec 定义 test-evidence-review v7 迁移后的目标能力；它只在本 change 内表达目标契约，不能作为当前测试、账本、主规范或已实现行为的证明。

## ADDED Requirements

### Requirement: 一个保留的最小原生测试入口对应一个 case
项目测试证据 MUST 以测试框架能够稳定独立选择、单独报告且拥有完整测试意图的最小原生测试入口为登记粒度。本次范围内每个保留入口 MUST 恰好对应一个 case，聚合容器和内部环节 MUST NOT 独立登记。

#### Scenario: Runner 报告多个原生测试节点
- **WHEN** 一个测试文件、suite、脚本或 CI job 聚合多个可区分的原生测试节点
- **THEN** 聚合对象只作为执行或定位容器
- **THEN** 每个本次保留的最小原生测试入口分别拥有一个 case

#### Scenario: Helper 只服务所属测试入口
- **WHEN** fixture、helper、mock、hook、断言或步骤只参与一个原生测试节点的最终判定
- **THEN** 它归入所属入口的证据背景
- **THEN** 它不得建立独立 case

#### Scenario: 工程校验不是测试入口
- **WHEN** lint、类型检查、schema、生成物一致性、安全扫描或 workspace profile 只执行工程校验
- **THEN** 该校验由自身 owner 和验证链路承接
- **THEN** 测试证据目录不得把命令、job 或结果登记为测试 case

### Requirement: Topic 目录与单 case Markdown 是权威源
项目 MUST 固定使用 `docs/test-evidence` 作为测试证据根目录，以受控 `test-evidence-topics.json` 定义稳定测试责任 topic，并 MUST 让每个合法 `<topic>/<slug>.md` 恰好保存一个 case。topic MUST NOT 改变 case 身份或测试粒度。

#### Scenario: 按 topic 定位 case
- **WHEN** 维护者查询一个已定义 topic
- **THEN** 结果只包含该 topic 直属目录中的合法 case
- **THEN** 每个结果的 sourcePath 使用 `<topic>/<slug>.md`

#### Scenario: 目录包含未知成员
- **WHEN** 根目录出现未知 topic、额外根文件、符号链接、嵌套目录或空 topic 目录
- **THEN** 严格检查报告阻断诊断
- **THEN** 非法布局不得通过只读索引回退绕过

### Requirement: Case 只表达入口、契约和证明
每个 case MUST 使用全目录唯一且稳定的 case ID，并 MUST 各有且只有一个 `Entry`、`Contract` 和 `Proves`。`Entry` MUST 精确定位同一个最小原生测试入口；`Contract` MUST 来自稳定 owner 语义；`Proves` MUST 描述直接可判断的可观察结果。目录 MUST NOT 使用 Status、Code、Verification、角色或源码 marker。

#### Scenario: 一个入口拥有多个定位
- **WHEN** 测试定义路径和 runner 精确选择命令都定位同一个原生测试节点
- **THEN** 同一 case 的 Entry 可以保存这些不重复定位
- **THEN** 这些定位不得跨越到另一个原生测试节点

#### Scenario: 旧聚合 case 需要拆分
- **WHEN** 旧 case 混合多个可独立命名、独立失败的测试意图
- **THEN** 迁移先拆成对应原生测试入口并分别建立 case
- **THEN** 只有语义连续且完整承接旧判断的入口可以保留旧 ID，其余入口使用唯一新 ID

#### Scenario: 历史回归缺少 owner 契约
- **WHEN** 旧 case 只描述历史事故、防止未来回归或内部实现路径，且没有当前 owner 明文语义
- **THEN** 该文字不得直接成为 Contract 或新增断言
- **THEN** 维护者缩小、转交或删除该证据项并在迁移映射中说明

### Requirement: 派生索引可重建且查询有界
项目 MUST 从受控 topic 表和全部合法 case Markdown 生成统一派生索引。索引 MUST 可删除重建，并 MUST 提供按 ID、精确 topic 和文本的有界查询以及单 case 原文展开；索引不得成为 case 内容或 topic 的第二权威源。

#### Scenario: 权威 case 变化使索引陈旧
- **WHEN** topic 定义、case 文件、路径或正文发生变化
- **THEN** 旧 source revision 被识别为陈旧
- **THEN** 严格检查失败，且同步入口可以从完整合法目录原子重建索引

#### Scenario: 查询使用合法内存投影
- **WHEN** 索引缺失或陈旧但 topic 表和全部 case 目录合法
- **THEN** list 或 show 可以返回带 warning 的只读内存投影
- **THEN** 查询不得隐式写回索引

### Requirement: 证据评估保持可观察与可靠
保留的测试 case MUST 指向当前契约背景、具体失败信号和调用方可观察结果，并 MUST 检查输入、fixture、mock、时序、随机性和环境不会使证据失真。测试 MUST NOT 只复述实现、只证明 mock，或让被测实现生成自身预期值。

#### Scenario: 测试证明公共失败语义
- **WHEN** 一个原生测试入口验证 owner 定义的错误或失败边界
- **THEN** Contract 指向该稳定边界
- **THEN** Proves 说明调用方可观察的错误、状态或资源结果

#### Scenario: 测试混合独立意图
- **WHEN** 一个原生测试节点包含可以独立命名和独立失败的多个测试意图
- **THEN** 维护者先拆分测试节点
- **THEN** 每个保留节点分别完成证据评估和 case 登记

### Requirement: 仓库内验证只使用单一测试证据源
项目 MUST 跟踪 test-evidence-review v7 的完整运行时分发内容，并 MUST 通过仓库内确定性入口严格检查固定目录。迁移完成后，集中账本、源码 marker、旧 case-catalog 采集器和兼容双读 MUST 不再作为活跃验证来源。

#### Scenario: Required profile 检查测试证据
- **WHEN** 本地或 CI 运行 required 文档验证
- **THEN** `cases` task 从项目跟踪的 v7 模块执行严格目录检查
- **THEN** 验证不依赖个人 skill、网络、updater 或旧 marker 采集

#### Scenario: 测试变更维护 case
- **WHEN** 任务新增、修改、删除或保留测试实现
- **THEN** 维护者按 runner 原生入口更新对应独立 case 和派生索引
- **THEN** 目标测试与目录严格检查共同作为交付证据

#### Scenario: 归档历史保留旧术语
- **WHEN** 已归档 OpenSpec change 记录迁移前的账本或 marker 流程
- **THEN** 历史 artifact 可以保持原文
- **THEN** 稳定文档、AGENTS、代码和 active changes 不得继续依赖旧流程
