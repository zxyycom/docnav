本 delta 的目标是定义 adapter-owned ref 的强制共享调用流程、非空 opaque string 载体、原样传递和正确性责任边界，并明确稳定 ref 错误和 adapter 专属文档所有权。当前 delta 已通过设计审计并解除实施门禁，尚未应用到现行主规范或实现。

## ADDED Requirements

### Requirement: 已实现 adapter 的私有行为必须有独立主文档
文档导航 MUST 为需要长期维护私有导航行为的已实现 adapter 指向独立主文档。

Markdown v0 adapter MUST 使用 `docs/adapters/markdown.md` 记录当前实现的导航行为、ref grammar、保证范围、错误分类和验证入口。`docs/references/markdown-navigator.md` MUST 只记录外部来源和迁移依据。

#### Scenario: 阅读 Markdown adapter 契约
- **WHEN** 实现者或审计者需要了解 Markdown adapter 的私有导航行为
- **THEN** 文档导航将其指向 `docs/adapters/markdown.md`
- **THEN** 共享 Ref、架构、协议和 adapter contract 文档只保留共享边界和链接

#### Scenario: 其它 adapter 建立自己的主文档
- **WHEN** 其它格式 adapter 需要长期维护格式私有行为
- **THEN** 该 adapter 使用自己的规范或专属文档记录这些行为
- **THEN** 其设计不继承 Markdown adapter 的 grammar、唯一性、稳定性或消歧语义

### Requirement: 稳定错误必须支持 adapter 报告非法 ref
原始协议和阅读错误层 MUST 支持稳定错误 `REF_INVALID`。该错误 MUST 表示请求中的 ref 是非空字符串且请求传输 shape 有效，但选定 adapter 无法按其当前私有 grammar 解释该 ref。

`REF_INVALID` 的稳定 details MUST 包含 `ref` 和 `reason`。具体 adapter MUST 自行决定哪些输入属于非法 grammar，并在其专属规范中说明 `REF_INVALID` 与其它 ref 错误的边界。

#### Scenario: Adapter 拒绝非法 ref grammar
- **WHEN** read 请求通过共享 schema 校验
- **AND** 选定 adapter 判定 ref 不符合其当前 grammar
- **THEN** adapter 返回 `REF_INVALID`
- **THEN** error details 包含原始 `ref` 和非空 `reason`

#### Scenario: 共享层不解析 adapter grammar
- **WHEN** `docnav` 或 MCP 接入层收到非空 ref
- **THEN** 共享层将 ref 原样传给选定 adapter
- **THEN** `docnav` 和 MCP 不自行判断该 ref 是否符合 adapter grammar

## MODIFIED Requirements

### Requirement: Ref 文档必须描述 adapter 拥有的 ref 边界
Ref 文档和示例 MUST 把 ref 描述为 adapter 生成和解释的非空 opaque string。共享协议、`docnav` 和接入层 MUST 只校验共享字段 shape，并将 ref 原样传给选定 adapter。

ref 的 grammar、定位或查询含义、适用 operation、读取粒度、唯一性、稳定性、消歧、多对一或一对多关系、文档变化后的行为，以及非法或未匹配 ref 的处理 MUST 由对应 adapter 的规范或专属文档定义。共享文档 MUST 通过链接指向对应主文档，不得复制 adapter 私有语义。

共享契约 MUST 强制保留 `outline/find -> ref -> read` 调用流程：adapter 在 outline 或 find 中生成 ref；调用方将相同 path 和 ref 原样提交给 read；core 根据 path 选择 adapter 并原样传递 ref；adapter 返回读取结果或稳定错误。

该流程保证不得解释为共享层保证 read 接受、完整消费、唯一定位、成功读取或返回特定区域。adapter 保留接受、拒绝和解释 ref 的全部权力，并在专属契约中定义结果语义。

该边界 MUST 表达为正确性责任分层，而不是放弃正确性。共享层 MUST 负责按 path 选择 adapter、保持 ref 原值并一致映射稳定错误；adapter MUST 负责其 ref 生成、解释、定位和失败行为符合自身公开契约。共享层 MUST NOT 在不了解 adapter grammar、文档状态和定位模型时替 adapter 建立读取成功或唯一定位保证。

#### Scenario: 共享层原样传递 ref
- **WHEN** 调用方把非空 ref 作为 read 参数提交
- **THEN** `docnav` 根据 path 选择 adapter 并原样传入 ref
- **THEN** `docnav`、MCP、共享协议和 schema 不解析或推断 ref 内部结构
- **THEN** adapter 按其自有契约解释或拒绝该 ref

#### Scenario: 共享调用链保持稳定
- **WHEN** 调用方取得 outline 或 find 返回的 ref
- **THEN** 调用方可以将相同 path 和 ref 原样提交给 read
- **THEN** core 选择 adapter 并原样传递 ref
- **THEN** adapter 返回读取结果或规范允许的稳定错误
- **THEN** 该流程不承诺 read 成功、唯一定位或返回特定区域

#### Scenario: 正确性责任按所有权分层
- **WHEN** 调用方将 outline 或 find 返回的 ref 提交给 read
- **THEN** core 按 path 选择 adapter、保持 ref 原值并一致映射 adapter 返回的稳定错误
- **THEN** adapter 按其专属契约生成、解释、定位或拒绝该 ref
- **THEN** 共享层不替 adapter 声明读取成功、唯一定位或特定区域保证

#### Scenario: 查找 adapter 私有 ref 语义
- **WHEN** 读者需要了解某个 adapter 的 ref 行为
- **THEN** 共享 Ref 文档将读者指向该 adapter 的主文档
- **THEN** 该 adapter 文档说明 grammar、适用 operation、保证范围和错误边界

#### Scenario: Adapter 可以选择不同定位保证
- **WHEN** 两个 adapter 为各自格式设计 ref
- **THEN** 一个 adapter 可以选择唯一定位，另一个 adapter 可以选择非唯一、部分消费或其它语义
- **THEN** 两者只需满足共享的非空字符串载体和原样传递边界

## REMOVED Requirements

### Requirement: Ref 可路由且唯一
**Reason**: 唯一定位、消歧和读取成功是 adapter 私有导航选择，不属于所有格式必须继承的共享能力。共享层只拥有非空字符串载体、adapter 所有权和原样传递边界。

**Migration**: 共享文档删除唯一定位和消歧保证；每个 adapter 在自己的规范或专属文档中明确实际 ref 语义、适用 operation、保证范围和错误分类。
