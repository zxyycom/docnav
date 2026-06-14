本 spec 只为 `plan-code-quality-observability` 定义未来非阻断代码质量观测能力的临时 requirements；当前不改变现有实现、主规范或验证链路。

## ADDED Requirements

### Requirement: 代码质量观测必须生成非阻断快照
Docnav 仓库的代码质量观测能力 MUST 生成当前源码状态的指标快照，并 MUST 将快照结果作为信息性报告处理。指标值本身 MUST NOT 使现有 workspace 验证、CLI 行为、adapter 行为、MCP 行为或 OpenSpec 验证失败。

#### Scenario: 本地生成快照
- **WHEN** 维护者运行未来的代码质量观测命令
- **THEN** 命令生成当前仓库源码的质量指标快照
- **THEN** 命令输出或写入报告位置
- **THEN** 文件行数、符号数量或函数复杂度的数值不作为失败条件

#### Scenario: 指标不改变现有验证语义
- **WHEN** 本 change 只存在于 `openspec/changes/plan-code-quality-observability/`
- **THEN** 当前 `verify:docnav-workspace`、Rust 测试、schema 验证和 smoke 验证行为保持不变
- **THEN** 当前主 specs、CLI、adapter、MCP、schema 和 examples 契约保持不变

### Requirement: 代码质量快照必须覆盖首期核心指标
代码质量观测快照 MUST 至少包含文件级行数、文件级符号数量、函数级行数、函数参数数量和函数圈复杂度。报告 MAY 包含额外指标，但额外指标 MUST 不改变首期核心指标的字段语义。

#### Scenario: 文件级指标
- **WHEN** 观测命令扫描一个纳入范围的源码文件
- **THEN** 快照记录该文件的规范化路径
- **THEN** 快照记录该文件的行数
- **THEN** 快照记录该文件的符号数量或可用工具能提供的等价声明数量

#### Scenario: 函数级指标
- **WHEN** 观测命令识别一个纳入范围的函数、方法或等价可调用单元
- **THEN** 快照记录该单元所属文件和名称
- **THEN** 快照记录该单元的行数
- **THEN** 快照记录该单元的参数数量
- **THEN** 快照记录该单元的圈复杂度或度量工具提供的等价 cyclomatic complexity 数值

### Requirement: 代码质量报告必须同时支持机器和人工消费
代码质量观测能力 MUST 从同一指标模型生成机器可读 JSON 和人类可读 summary。JSON MUST 保留完整明细和聚合字段；summary MUST 展示最需要人工关注的排名和分组信息。

#### Scenario: 生成 JSON 明细
- **WHEN** 观测命令完成扫描
- **THEN** 机器可读输出包含扫描元数据、文件指标、函数指标和聚合指标
- **THEN** 输出路径使用仓库相对路径或规范化路径
- **THEN** 输出不包含依赖目录、构建产物或缓存目录的指标记录

#### Scenario: 生成 Markdown summary
- **WHEN** 观测命令完成扫描
- **THEN** 人类可读 summary 展示按行数排序的文件列表
- **THEN** summary 展示按符号数量排序的文件列表
- **THEN** summary 展示按圈复杂度排序的函数列表
- **THEN** summary 展示按函数行数或参数数量排序的函数列表

### Requirement: 代码质量观测必须限定扫描边界
代码质量观测能力 MUST 默认扫描仓库维护的 Rust 源码和 JavaScript 脚本，并 MUST 排除依赖、构建产物、虚拟环境、发布产物和缓存目录。未来扩展到文档、schema 或其它语言时，MUST 明确新增路径范围和指标语义。

#### Scenario: 默认排除非源码产物
- **WHEN** 观测命令遍历仓库
- **THEN** 默认排除 `.git`、`target`、`node_modules`、`.venv`、`dist`、`build` 和缓存目录
- **THEN** 报告不把这些目录中的文件计入文件排名、函数排名或聚合指标

#### Scenario: 按语言或路径分组
- **WHEN** 快照包含多种语言或多个代码区域
- **THEN** 报告保留语言或路径分组信息
- **THEN** 汇总结果能区分 Rust crates、仓库脚本和其它未来纳入范围

### Requirement: 代码质量观测必须为未来报告和门禁保留演进边界
代码质量观测能力 MUST 将当前快照、未来自动报告和未来门禁策略分离。任何阈值、baseline、PR 差异阻断或豁免机制 MUST 由后续 change 明确定义，不得从首期观测结果隐式推导。

#### Scenario: 自动报告可复用快照
- **WHEN** 未来 CI 或本地工作流需要生成自动报告
- **THEN** 它可以复用当前观测快照的 JSON 数据模型
- **THEN** 自动报告可以展示趋势、top N 或 changed files 指标
- **THEN** 自动报告不得在未定义门禁策略时因指标值失败

#### Scenario: 未来门禁需要单独定义
- **WHEN** 后续 change 提议让代码质量指标阻断合并或验证
- **THEN** 该 change MUST 明确阈值或 baseline 来源
- **THEN** 该 change MUST 明确豁免、历史债务处理和 changed files 策略
- **THEN** 该 change MUST 明确失败输出和修复指引
