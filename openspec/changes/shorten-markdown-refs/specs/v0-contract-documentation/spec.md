本 spec delta 仍处于审核阶段。

## ADDED Requirements

### Requirement: 已实现 adapter 的私有行为必须有独立主文档
文档导航 MUST 为需要长期维护私有导航行为的已实现 adapter 指向独立主文档。

Markdown v0 adapter MUST 使用 `docs/adapters/markdown.md` 记录当前实现的导航行为、保证范围和验证入口。`docs/references/markdown-navigator.md` MUST 只记录外部来源和迁移依据。

#### Scenario: 阅读 Markdown adapter 契约
- **WHEN** 实现者或审计者需要了解 Markdown adapter 的私有导航行为
- **THEN** 文档导航将其指向 `docs/adapters/markdown.md`
- **THEN** 共享 Ref、架构、协议和 adapter contract 文档只保留所有权摘要和链接

#### Scenario: 其它 adapter 建立自己的主文档
- **WHEN** 其它格式 adapter 需要长期维护格式私有行为
- **THEN** 该 adapter 使用自己的规范或专属文档记录这些行为
- **THEN** 其设计不继承 Markdown adapter 的私有语义

## MODIFIED Requirements

### Requirement: Ref 文档必须描述 adapter 拥有的 ref 边界
Ref 文档和共享示例 MUST 将 ref 描述为 adapter 生成和消费的非空 opaque string。共享协议、`docnav` 和接入层 MUST 原样传递 ref。

ref 的具体语法、定位方式、读取行为和保证范围 MUST 由对应 adapter 的规范或专属文档定义。共享文档 MUST 通过链接指向对应主文档，不得复制 adapter 私有语义。

#### Scenario: 从 outline 执行 read
- **WHEN** 调用方从 outline 或 find 取得 ref
- **THEN** 调用方将相同 path 和 ref 原样传给 read
- **THEN** `docnav` 根据 path 选择 adapter 并原样传入 ref
- **THEN** adapter 按其自有契约消费 ref

#### Scenario: 查找 adapter 私有 ref 语义
- **WHEN** 读者需要了解某个 adapter 的 ref 行为
- **THEN** 共享 Ref 文档将读者指向该 adapter 的主文档
- **THEN** core 和 MCP 不解析或推断 ref 结构

## REMOVED Requirements

### Requirement: Ref 可路由且唯一
**Reason**: adapter 所有权和原样传递属于共享契约；具体定位方式和保证范围属于 adapter 私有契约。

**Migration**: 共享文档改用 adapter-owned opaque ref 表述；各 adapter 在自己的规范或专属文档中定义具体行为。
