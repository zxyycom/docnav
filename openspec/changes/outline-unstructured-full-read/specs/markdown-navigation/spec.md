本 delta 定义 core `docnav outline` dispatch 到 linked Markdown adapter 时对非结构化 outline 全文读取策略的支持；它只在 `openspec/changes/outline-unstructured-full-read/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## ADDED Requirements

### Requirement: Markdown outline 支持配置触发的非结构化全文读取
Linked Markdown adapter outline handling MUST 支持生效配置中的非结构化 outline 策略。命中该策略时，Markdown outline MUST 直接返回整篇 Markdown 原文内容，并 MUST NOT 返回 heading entries、`doc:full` entry、ref、page 或 continuation。本 change 不定义具体配置文件、格式或合并方式。

#### Scenario: Core CLI dispatch 到 Markdown linked adapter 后自动全文读取
- **WHEN** 调用方执行 `docnav outline docs/raw-note.md`
- **AND** 生效 Markdown adapter 配置声明该 path 为非结构化文档
- **THEN** 输出为非结构化 outline 结果
- **THEN** content 等于整篇 Markdown 原文
- **THEN** content_type 为 `text/markdown`
- **THEN** 结果不包含 heading entries、`doc:full`、ref 或 page

#### Scenario: 无配置时保留 doc:full fallback
- **WHEN** 当前 outline 参数过滤后没有可见 heading
- **AND** 目标 path 未命中非结构化 outline 配置
- **THEN** Markdown outline 仍返回 ref 为 `doc:full` 的单条 entry
- **THEN** 使用该 ref 执行 read 返回整篇 Markdown 文档

### Requirement: Markdown 非结构化 outline 必须被 smoke 覆盖
Markdown adapter smoke 和 fixture corpus MUST 覆盖至少一个配置命中的非结构化 Markdown 文件，并验证 readable-json、readable-view 和 protocol-json 三种输出模式的非结构化 outline shape。

#### Scenario: Smoke 覆盖非结构化 outline shape
- **WHEN** smoke suite 对配置命中的 Markdown fixture 执行 outline
- **THEN** readable-json 和 protocol-json 结果不包含 entries、ref 或 page
- **THEN** readable-json 使用 documented outline success payload 分支
- **THEN** readable-view 使用 `/content` block 承载全文
- **THEN** 三种输出都包含稳定原因字段或等价可读说明
