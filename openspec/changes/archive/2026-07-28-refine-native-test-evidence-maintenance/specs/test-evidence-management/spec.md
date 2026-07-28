本 delta 为测试证据能力增加目录驱动发现和独立的 Entry/Claim 粒度约束；它是临时 change 规范，不表示实现已经满足要求。

## ADDED Requirements

### Requirement: Runner profile 通过目录规则完整展开测试面

项目 MUST 让普通 runner 测试面由受控 source roots 与 include/ignore patterns 展开，并 MUST 只用显式 supplemental files 补充目录规则之外的特殊文件。展开后的同一确定性文件集合 MUST 同时用于静态入口发现和 runtime runner report。

#### Scenario: 新增普通 Bun 测试文件

- **WHEN** 维护者在受控 source root 下新增匹配 include 且不匹配 ignore 的普通测试文件
- **THEN** strict check 自动把该文件交给静态扫描与 Bun runner
- **THEN** 其中的原生入口不能因未手写文件名而避开 machine inventory 闭合

#### Scenario: 目录规则忽略文件

- **WHEN** 一个文件匹配 include 同时匹配显式 ignore pattern
- **THEN** profile 展开不把该文件交给静态扫描或 runner
- **THEN** ignore 规则作为受控测试面的一部分保持可审计

#### Scenario: 特殊文件补充

- **WHEN** 一个安全的普通测试文件不属于任何目录规则但列入 supplemental files
- **THEN** profile 展开把它加入静态与 runtime 的共同文件集合
- **THEN** 已由目录规则纳入的文件不得再作为冗余补充通过验证

#### Scenario: profile 边界无效

- **WHEN** source root、pattern 或 supplemental path 不安全、pattern 使用 `!` / `#` 控制语法、配置路径经过符号链接、目标类型错误、include 无匹配或最终集合为空
- **THEN** strict check 返回 blocking profile diagnostic
- **THEN** wrapper 不以部分集合继续生成 inventory

### Requirement: Entry 粒度与 Claim 信息增量独立决定

项目 MUST 在 runner 能独立选择和报告的范围内按完整、可归因的测试意图维护 Entry，并 MUST 让 smoke Entry 的 fingerprint 覆盖其 leaf task 与 source roots 内可达的自有实现；项目 MUST 只为无法从 owner requirement 与入口事实直接恢复的长期判断维护 Claim。Claim MAY 关联一个或多个当前 Entry，Entry MAY 没有 Claim。

#### Scenario: 一个 leaf 混合独立意图

- **WHEN** 一个测试 leaf 同时验证可以独立命名、独立失败且属于不同契约边界的意图
- **THEN** 维护者把它拆成可独立报告的原生入口
- **THEN** 每个新入口分别生成 machine case

#### Scenario: 多步骤共同证明一个不变量

- **WHEN** 多个命令步骤、输出模式或代表输入共同构成一次不可分割的 round trip、parity 或 precedence 判断
- **THEN** 它们可以保留为一个拥有单一失败归因的 Entry
- **THEN** 系统不得按断言或步骤数量机械拆分

#### Scenario: Smoke case 的可达 helper 实现变化

- **WHEN** smoke leaf 的 `run` 实现或 source roots 内可达的顶层 helper 声明发生变化
- **THEN** 对应 Entry 的 source fingerprint 变化
- **THEN** 同一模块中不可达的其它声明不使该 Entry 产生 `implementation-changed`

#### Scenario: 一个长期判断需要多个入口

- **WHEN** 多个当前 Entry 分别观察同一 owner requirement 的不同必要边界
- **THEN** 一个高信息 Claim 可以在 `supportedBy` 中关联这些 Entry
- **THEN** statement 与 observations 不得超出这些入口能够直接观察的范围

#### Scenario: 候选 Claim 没有信息增量

- **WHEN** Claim 只复述 owner requirement、入口名称或通用测试质量模板
- **THEN** 维护者删除或拒绝该 Claim
- **THEN** 对应 Entry 继续作为合法 machine case 保留

### Requirement: 派生账本可以从陈旧状态重建

项目 MUST 允许 sync 在测试源码、profile 或 Claim 已变化而 committed inventory/index 尚未更新时执行完整 runner 发现。验证派生状态新鲜度的检查 MUST 发生在 runner report 产生和派生制品重建之后，不得形成要求旧 inventory 预先新鲜的自举循环。

#### Scenario: 测试入口发生 rename 或 split

- **WHEN** 当前源码中的测试入口已经变化而 committed inventory 仍记录旧入口
- **THEN** sync 仍可执行测试证据工具自身的 runner tests 并取得完整 report
- **THEN** 重建后的 strict check 双向验证新 inventory、Claim 和 index
