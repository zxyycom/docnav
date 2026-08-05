# decision-record-management Specification

## Purpose
定义项目级长期决策的权威来源、生命周期、owner 分工、演进事务与可复现验证，并区分当前基线、已确认未来方向和实施 change 的作用。
## Requirements
### Requirement: 决策领域与记录是权威来源
项目级决策集合 MUST 使用受控领域表和各自 Markdown 记录作为领域、内容、生命周期、对齐与关系的权威来源；派生索引 MUST 能从这些来源删除后重建，且 MUST NOT 反向拥有或补造决策状态。

#### Scenario: 从权威来源重建索引
- **WHEN** 领域定义或已建立决策 Markdown 发生合法变化
- **THEN** 维护入口从完整权威来源重建统一索引
- **THEN** 严格检查确认索引成员、metadata、关系和 source revision 与来源一致

#### Scenario: 陈旧索引不能提供当前判断
- **WHEN** 持久化索引缺失、损坏或与权威来源不一致
- **THEN** 查询或严格检查明确报告索引问题
- **THEN** 系统不得从陈旧索引推断当前领域、生命周期、对齐、正文或关系

### Requirement: 稳定身份与正文语义保持单一
每条决策 MUST 以 `<domain-id>/<semantic-slug>.md` 作为稳定身份，并 MUST 在 frontmatter 与依次排列的 `目的`、`背景`、`决策` 正文中完整表达自身当前判断。摘要 MUST 只投影正文语义，路径、状态、建立时间和关系 MUST NOT 在正文建立第二份状态。

#### Scenario: 读取一条已建立决策
- **WHEN** 调用方按稳定路径展开决策
- **THEN** frontmatter 提供标题、生命周期、对齐、建立时间、摘要和直接关系
- **THEN** 正文可以独立恢复目的、形成背景和采用方向

#### Scenario: 结构迁移保持既有身份
- **WHEN** 决策存储格式或索引实现发生迁移且判断语义未改变
- **THEN** 已建立记录的稳定路径、建立时间、生命周期和真实直接关系保持不变
- **THEN** 迁移不得伪装成新的决策演进

### Requirement: 生命周期、对齐与演进使用显式事务
项目级决策维护 MUST 区分未激活候选、活动已对齐、活动未对齐和已归档状态。激活、演进、标记对齐、归档和丢弃 MUST 使用对应显式维护动作；关系 MUST 只表达从新判断到直接前序的真实演进。

#### Scenario: 已确认方向尚未成为当前事实
- **WHEN** 已确认决策表达未来方向但完整当前事实尚未实现该方向
- **THEN** 记录保持 `active + unaligned`
- **THEN** 相关工作不必提前准备或实现，但必须注意不要主动增加障碍
- **THEN** 在既有授权范围内能够顺手实现时可以一并完成

#### Scenario: 未对齐状态不授权推进
- **WHEN** 一条活动未对齐决策适用于当前工作，但当前请求或既定任务没有要求实施该方向
- **THEN** agent 不得仅因该决策的活动或对齐状态而扩大任务范围、提高当前实施优先级或提前建设实现
- **THEN** 决策正文仍可记录未来方向和先后关系
- **THEN** 未缩小事实差距本身不构成一致性问题

#### Scenario: 对齐基线经过事实核对
- **WHEN** 维护者将完整活动决策与当前事实来源逐项核对且目标已满足
- **THEN** `mark-aligned` 建立单向 `active + aligned` 基线
- **THEN** 后续事实偏离被报告为一致性问题，而不是把原记录改回未对齐

#### Scenario: 新判断演进直接前序
- **WHEN** 一条已确认的新决策修订、替代、判定无效或归并活动直接前序
- **THEN** 一次可恢复事务归档全部直接前序、写入真实直接关系、激活新记录并重建索引
- **THEN** 新活动记录能够脱离前序独立表达当前判断

### Requirement: 决策管理保持 owner 分工
决策记录 MUST 只保存已确认且跨 change 仍有长期影响和回放价值的方向与重要细节。已经成为当前基线的稳定行为与 public contract MUST 由对应 owner 文档承接，只服务一次实施的判断 MUST 留在对应 OpenSpec change，当前实现状态 MUST 由代码、测试和 release artifact 证明。

#### Scenario: Change 内判断不自动进入全局记录
- **WHEN** 一项已确认判断只约束当前 active change
- **THEN** 该判断保存在 change 的 design Decisions
- **THEN** change 归档不得自动创建全局决策记录

#### Scenario: 决策与事实来源失配
- **WHEN** 已对齐决策与当前 owner 文档或实现证据发生偏离
- **THEN** 维护者报告一致性问题并按 owner 分工同步正确来源
- **THEN** 决策索引不得被当作当前产品行为的证明

### Requirement: 仓库内严格验证是可复现门禁
项目 MUST 跟踪 decision-records v5 的完整运行时分发内容，并 MUST 通过仓库内确定性入口严格验证决策集合。required 验证 MUST NOT 依赖个人 skill 目录、浮动网络资源或执行 updater。

#### Scenario: Required profile 验证决策集合
- **WHEN** 本地或 CI 运行 required 文档验证
- **THEN** validator 从仓库跟踪路径调用 v5 校验实现
- **THEN** 非法领域、Markdown、生命周期、对齐、关系或陈旧索引使该任务失败

#### Scenario: 离线环境运行验证
- **WHEN** 工作区已完成正常 clone 且没有个人 skill 安装或网络
- **THEN** 决策严格检查仍可执行
- **THEN** updater 和 release 查询不进入验证链路
