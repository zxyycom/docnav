# markdown-reference-baseline Specification

## Purpose
定义 MarkdownNavigator 作为 Markdown 行为参考基线的来源、复验方法、迁移决策和边界案例要求。
## Requirements
### Requirement: 记录 MarkdownNavigator 来源和复验方法
Markdown 参考文档 MUST 记录参考项目路径、提交、命令入口和可复验行为边界，并明确其不是 Docnav 兼容目标。

#### Scenario: 复验参考来源
- **WHEN** 实现者查看 Markdown 行为来源
- **THEN** 文档能够定位参考仓库、提交和复验方法

### Requirement: 记录 Markdown 行为迁移决策
Markdown 参考文档 MUST 为 heading、章节范围、frontmatter、代码围栏、重复项、编码、默认限制和 page 标注迁移决策。

#### Scenario: 评估旧行为
- **WHEN** 实现者查看参考 CLI 行为
- **THEN** 文档明确该行为在 Docnav 中保留、调整、推迟或移除

### Requirement: 保留成熟 parser 行为基线
Markdown 适配器 MUST 使用成熟 parser；章节 MUST 从目标 heading 开始，并在下一个同级或更高级 heading 前结束。

#### Scenario: 读取包含子章节的章节
- **WHEN** read 选择包含更深层 heading 的章节
- **THEN** 结果包含子章节并在下一个同级或更高级 heading 前结束

### Requirement: Markdown Outline 扁平且有限
Markdown outline MUST 返回扁平 ref/display entries，内置默认 MUST 为每页最多 6000 字符且只展示 H1-H3。

#### Scenario: 嵌套 heading
- **WHEN** 文档包含 H1、H2 和 H3
- **THEN** outline 返回按文档顺序排列的扁平条目
- **THEN** 每项 ref 使用 Markdown heading path 表达层级
- **THEN** display 只保留 ref 之外的紧凑信息

### Requirement: Markdown Read 有限且可继续
Markdown read 内置默认 MUST 为每页最多 6000 字符，并 MUST 返回下一页 page 或 null。

#### Scenario: Read 超过默认限制
- **WHEN** 章节超过默认字符预算
- **THEN** read 返回有限内容和下一页 page

### Requirement: 重复项生成唯一 Ref
重复标题和重复完整路径 MUST 生成不同 ref；read MUST NOT 通过最近行静默消歧。

#### Scenario: 重复完整路径
- **WHEN** 文档包含重复完整 heading path
- **THEN** outline 为每项生成不同 ref

### Requirement: 复用 Markdown 边界案例
Markdown 适配器测试 MUST 覆盖无 heading、仅深层 heading、无效 heading、frontmatter、代码围栏、重复标题、重复路径、深层章节和非 UTF-8。

#### Scenario: 规划适配器测试
- **WHEN** 实现者制定或更新 Markdown 测试计划
- **THEN** 测试计划包含全部参考边界案例
