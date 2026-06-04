## ADDED Requirements

### Requirement: 记录 MarkdownNavigator 来源和验证状态
Markdown 参考文档 MUST 记录参考项目路径、参考版本或提交、已执行验证以及参考范围，且 MUST 明确参考项目不是 Docnav 的协议兼容目标。

#### Scenario: 审计参考基线
- **WHEN** 审计者查看 Markdown 行为来源
- **THEN** 文档能够定位到 `D:\project\skills\MarkdownNavigator`
- **THEN** 文档说明参考项目黑盒测试的验证结果和时间

### Requirement: 记录 Markdown 结构行为迁移决策
Markdown 参考文档 MUST 为 heading 识别、章节范围、深层 heading、frontmatter、代码围栏、重复标题、重复路径、编码和截断行为分别标注“保留、调整、推迟或移除”及理由。

#### Scenario: 评估重复完整路径
- **WHEN** Markdown 中存在两个相同的完整 heading 路径
- **THEN** 参考文档说明 MarkdownNavigator 的现有行为
- **THEN** 参考文档说明 Docnav 不允许 read 通过最近行静默消歧

### Requirement: 保留成熟 parser 行为基线
Markdown 参考文档 MUST 要求后续 Markdown 适配器使用成熟 parser，并记录章节从目标 heading 开始、在下一个同级或更高级 heading 前结束的基线行为。

#### Scenario: 代码围栏包含伪 heading
- **WHEN** Markdown 代码围栏中出现以 `#` 开头的文本
- **THEN** 参考基线要求该文本不进入 outline

#### Scenario: 读取包含子章节的章节
- **WHEN** read 选择一个包含更深层 heading 的章节
- **THEN** 参考基线要求结果包含其子章节
- **THEN** 结果在下一个同级或更高级 heading 前结束

### Requirement: 明确不继承不稳定 CLI 契约
Markdown 参考文档 MUST 明确 Docnav 不直接继承 MarkdownNavigator 的列数组 headings 输出、自由文本路径 selector、近似行号消歧、非结构化错误、未知参数忽略和无 envelope 文本输出。

#### Scenario: 对照旧 CLI 与 Docnav 协议
- **WHEN** 实现者查看 MarkdownNavigator 的 `headings` 或 `section` 输出
- **THEN** 参考文档指出这些输出只能用于理解产品行为
- **THEN** 实现者被引导使用 Docnav 协议和 selector 文档定义机器契约

### Requirement: 复用 Markdown 边界案例清单
Markdown 参考文档 MUST 将参考项目中已验证的关键 fixtures 和黑盒场景整理为后续 Markdown 适配器的测试输入清单。

#### Scenario: 规划 Markdown 适配器测试
- **WHEN** 后续变更创建 Markdown 适配器测试计划
- **THEN** 测试计划包含无 heading、仅深层 heading、无效 heading、frontmatter、重复标题、重复路径和深层章节等参考场景
