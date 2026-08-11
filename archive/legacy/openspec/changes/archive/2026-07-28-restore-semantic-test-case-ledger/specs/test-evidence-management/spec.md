本临时 delta spec 的目标是把 `test-evidence-management` 改为 scanned test entity（测试实体）与 Topic/Case 语义账本的双向覆盖契约；它不表示该目标已经实现。

## ADDED Requirements

### Requirement: 完整当前树生成测试实体集合
项目 MUST 让受控 runner profile 从完整当前树发现测试框架能够稳定独立选择或报告、且拥有完整测试意图的最小 scanned test entity（测试实体），并 MUST 在内存中闭合 static declaration、runtime report 与实体映射。每个测试实体 MUST 具有唯一且确定性的 entity key；扫描结果 MUST NOT 通过手写 Entry、`NativeTestEntry` 长期模型或 committed inventory 成为第二事实源。

#### Scenario: Static 与 runtime 集合闭合
- **WHEN** strict check 执行 supported runner profile
- **THEN** scanner 双向比较 static、runtime 与实体映射并生成唯一当前测试实体集合
- **THEN** 任一 `static-only`、`runtime-only`、unsupported shape 或 duplicate test entity 都阻断检查

#### Scenario: 聚合和内部环节不是实体
- **WHEN** 测试文件、suite、runner、CI job、fixture、helper、mock、hook、断言或步骤只聚合或服务测试节点
- **THEN** scanner 只为可独立选择或报告的最小完整测试节点生成测试实体
- **THEN** 聚合或内部环节不得获得 entity key

#### Scenario: 工程校验不是测试实体
- **WHEN** lint、类型检查、schema、生成物一致性、安全扫描或 workspace profile 只执行工程校验
- **THEN** 该校验由自身 owner 和验证链路承接
- **THEN** test Case 账本不得把命令、job 或结果登记为测试实体

### Requirement: Topic catalog 与 Topic 文件直接拥有语义 Case
项目 MUST 固定使用 `docs/testing/cases/topics.json` 定义稳定 topic ID、说明和顺序，并 MUST 让每个受控 topic 恰有一个 `docs/testing/cases/<topic>.md`，保存该 topic 的零个或多个当前 Case。Case root MUST 是 workspace 内的非符号链接目录；`topics.json` 以及受控或未知的 `.md` 成员 MUST 解析为 workspace 内的非符号链接普通文件。Case ID MUST 在全部 topic 中全局唯一且稳定。项目 MUST NOT 维护一 Case 一文件目录或重复 Case/entity 关系的 committed query index。

#### Scenario: 空 topic 仍然稳定存在
- **WHEN** 一个已定义 topic 暂时没有 Case
- **THEN** `topics.json` 和对应 `<topic>.md` 仍共同定义合法空 topic
- **THEN** `topics` 查询返回该 topic 且 strict check 不因它为空而失败

#### Scenario: 按 topic 定位 Case
- **WHEN** 维护者按精确 topic 查询账本
- **THEN** 结果只包含该 topic 文件中的 Case
- **THEN** topic 不改变 Case ID、证明粒度或实体身份

#### Scenario: Case source 保持 workspace-safe
- **WHEN** Case root、`topics.json` 或任一 `.md` 成员越出 workspace、经过符号链接或不是其要求的目录/普通文件类型
- **THEN** strict check 报告阻断诊断
- **THEN** 查询不得通过缓存或兼容读取绕过非法布局

#### Scenario: Case 目录成员边界
- **WHEN** Case 目录包含嵌套目录、任意符号链接或未在 `topics.json` 中登记的 `.md` 文件
- **THEN** strict check 报告阻断诊断
- **THEN** 与 Case source 无关的非 Markdown 普通文件被忽略

#### Scenario: Topic Markdown 只接受受控语法
- **WHEN** parser 读取一个受控 topic Markdown
- **THEN** 文件只可包含一个 H1，随后是空行和合法的 `## Case <CASE-ID>: <title>` Case blocks
- **THEN** 仅含 H1 的空 topic 合法，而 malformed Case H2、其它 H2 或 Case block 外 prose 均报告阻断诊断

### Requirement: Case 直接表达单一 Owner、证明和当前测试实体
账本 MUST 只保存当前 implemented Case。每个 Case MUST 恰好声明一个精确定位当前 Markdown heading 的 `Owner`、非空 `Entities` 和非空 `Proves`；该 Owner MUST 真正拥有全部 `Proves` 所述责任，而不只是与主题相关，每条 `Proves` MUST 描述该 Owner 下责任方可观察的判断，每个 entity key MUST 完整、精确地来自当前 scanner。稳定 Case ID MUST 持续代表同一语义责任；已退休 ID MUST NOT 被重新用于其它语义。Case MUST NOT 保存 `Status`、Entry、Claim、source fingerprint、Code path、Verification、源码 marker 或派生反向引用。

#### Scenario: Case 关联当前测试实体
- **WHEN** 维护者登记一个当前 Case
- **THEN** `Entities` 至少包含一个当前测试实体的 entity key
- **THEN** 每个列出的测试实体都能直接产生该 Case 所述的可观察证明信号

#### Scenario: Planned 行为不进入当前 Case 账本
- **WHEN** 一个行为仍是 planned 且没有当前实现证据
- **THEN** 该行为留在 OpenSpec、owner 文档或其它规划 owner
- **THEN** Case 账本不得用 `Status` 或空 `Entities` 建立占位 Case

#### Scenario: 历史 Case 跨越多个 owner
- **WHEN** 一个历史 Case 的证明陈述不能由单一当前 owner 完整承接
- **THEN** 维护者拆分或收窄 Case，并为每个保留 Case 选择一个精确 Owner
- **THEN** `Proves` 不得超出该 Owner 与当前测试实体能够直接支持的范围

#### Scenario: 历史 implemented Case 有起点直接证据
- **WHEN** 逐项审查的历史 implemented Case 所述生产契约当前仍然存在，且本 change 实现开始前已有当前测试实体直接产生该语义的可观察信号
- **THEN** 维护者保留语义连续的 Case ID，并按当前事实迁移 Owner、Entities 与 Proves
- **THEN** 不得仅因实现路径或测试实体改变而换用新 ID

#### Scenario: 当前能力缺少起点直接测试实体
- **WHEN** 历史 implemented Case 所述生产能力仍存在，但本 change 实现开始前没有当前测试实体直接产生该语义的可观察信号
- **THEN** 该 seed 不迁入 current ledger，也不成为本 change 新增或改写产品测试的义务
- **THEN** 独立的 owner-driven product test change 决定是否补足直接测试，历史 Case ID 保持原语义且不得换义复用

#### Scenario: 历史生产能力已经移除
- **WHEN** owner 与当前源码证明确认一个历史 implemented Case 所述生产能力已经移除
- **THEN** 维护者以明确的 owner/source 依据退休该 Case，而不是编造空 Case
- **THEN** 该 Case ID 不得被重新用于其它语义

### Requirement: 当前测试实体与 Case 双向覆盖
严格检查 MUST 计算当前测试实体与 Case 的 many-to-many 关系，并 MUST 保证每个当前测试实体至少属于一个 Case、每个 Case 至少引用一个当前测试实体。项目允许同一测试实体支持多个 Case、同一 Case 由多个测试实体支持，并 MUST NOT 强制一对一映射。

#### Scenario: 当前测试实体没有语义 Case
- **WHEN** scanner 返回的一个当前测试实体未被任何 Case 精确列举
- **THEN** strict check 报告 uncovered test entity 并失败
- **THEN** 维护者必须把它归入真实证明目标或调整测试，而不是生成模板 Case

#### Scenario: Case 没有当前测试实体
- **WHEN** Case 的 `Entities` 为空或全部 key 均不在当前 scanner 集合
- **THEN** strict check 报告无当前实现证据并失败
- **THEN** 维护者必须更新 Case、实体关系或当前测试

#### Scenario: 一个测试实体支持多个 Case
- **WHEN** 同一测试实体直接观察多个可独立命名的 owner 结果
- **THEN** 多个 Case 可以引用同一个 entity key
- **THEN** strict check 分别验证各 Case 的 Owner 与证明内容，不把关系复用诊断为重复测试实体

### Requirement: 查询和 Required 门禁只使用当前证据链
项目 MUST 让 `topics`、有界 `list` 和单 Case `show` 直接读取 topic catalog 与 Topic/Case 文件，并 MUST 让仓库内 `check` 从完整 scanner test entity 集合与同一 Case source 执行严格验证。迁移完成后，Evidence Claim、hand-written Entry、`NativeTestEntry` 长期模型、native inventory、query index、旧源码 marker、`sync`、baseline `changes` 与兼容双读 MUST NOT 作为活跃验证或查询来源。

#### Scenario: 只读查询不生成派生状态
- **WHEN** 维护者按 topic、Owner、entity key 或文本执行有界 `list`，或通过单 Case `show` 精确查询 Case ID
- **THEN** 查询直接从合法 `topics.json` 与 topic files 返回有界结果
- **THEN** 查询不执行隐式写入且不要求 committed inventory/index

#### Scenario: Case ID 只通过单 Case show 查询
- **WHEN** 维护者需要按完整 Case ID 精确定位 Case
- **THEN** CLI 只通过 `show <CASE-ID>` 执行该查询
- **THEN** `list` 只接受 topic、Owner、entity key、文本与 pagination 的有界过滤，不提供 Case ID filter

#### Scenario: Required profile 检查完整当前树
- **WHEN** 本地或 CI 运行 workspace required profile
- **THEN** test-evidence check 先证明 static/runtime/实体映射闭合，再证明 Topic/Case 结构与双向 coverage
- **THEN** missing、unknown、duplicate 或 malformed Case/test-entity 关系使 required check 失败

#### Scenario: 历史材料不参与当前验证
- **WHEN** Git 或已归档 OpenSpec change 保留旧账本、Claim、Entry、inventory、index 或 marker 记录
- **THEN** 历史 artifact 可以保持原文
- **THEN** 当前 docs、skill、代码、测试和 active change 不得读取它们来补足当前 coverage

## REMOVED Requirements

### Requirement: 一个保留的最小原生测试入口对应一个 case
**Reason**: 入口事实与语义 Case 不是一对一关系；强制每入口一个 case 会把 scanner 粒度误当成证明粒度。

**Migration**: scanner 继续生成当前测试实体集合，语义 Case 通过 `Entities` 建立 many-to-many 关系并由双向 coverage 验证。

### Requirement: Topic 目录与单 case Markdown 是权威源
**Reason**: 一 Case 一文件增加碎片，旧 topic 目录规则又错误地拒绝空 topic。

**Migration**: 使用 `docs/testing/cases/topics.json` 定义受控稳定 topic，并由每个 `<topic>.md` 保存零个或多个当前 Case。

### Requirement: Case 只表达入口、契约和证明
**Reason**: `Entry` / `Contract` 的单入口模型被 owner、Proves 与精确 entity keys 的语义 Case 取代。

**Migration**: 把 101 个 historical implemented Case 作为逐项 review seed：只有本 change 实现开始前已有当前测试实体直接支持的语义才保留连续 ID 并迁移当前 Owner、Proves 与支持实体；生产能力仍存在但缺这种直接实体时不迁移且不反向补产品测试，生产能力已移除时以明确 owner/source 依据退休；未迁移或退休 ID 均不得换义复用，historical planned Case 不迁入当前账本。

### Requirement: 派生索引可重建且查询有界
**Reason**: committed index 重复 Topic/Case 内容与实体关系，形成不必要的新鲜度状态。

**Migration**: `topics`、`list` 与 `show` 直接解析 topic catalog/files；`check` 按需连接当前 scanned test entities，不再运行 index sync。

### Requirement: 证据评估保持可观察与可靠
**Reason**: 可观察性和可靠性义务仍然需要，但其 owner 从单入口 Case/Contract 模型迁移到语义 Case 的单一 Owner/Proves。

**Migration**: 由新增的 Case 字段契约和双向 coverage requirements 完整承接，并在测试变更审查中继续核对 fixture、mock、时序和独立预期值。

### Requirement: 仓库内验证只使用单一测试证据源
**Reason**: 原 requirement 固定 v7 Entry/Claim/派生 index 链，已不再是目标单一证据源。

**Migration**: required profile 只从项目 scanner 与 Topic/Case 文件执行新严格检查；旧模型只留在历史 artifact。
