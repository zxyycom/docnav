# 文档模式

## Inline Documentation

写注释的条件：

1. 解释非显而易见的业务、性能、安全、兼容性或生命周期约束。
2. 解释为什么没有使用更常见的写法。
3. 指向完整 ADR、issue、spec 或 source citation。
4. 记录错误处理、边界条件或顺序依赖。

不要写只复述代码的注释。不要用注释保存旧代码。TODO 只适合短期、可定位、可验收的后续工作；能立即修的内容直接修。

## API Documentation

public API 文档至少说明：

1. 输入字段、默认值、稳定性和约束。
2. 返回结构、分页、continuation、错误类型和 side effects。
3. 兼容性承诺和破坏性变化。
4. 最小可运行示例。
5. 与 schema、examples、tests 或 official docs 的链接。

TypeScript、Rust types、OpenAPI、JSON Schema 和 generated docs 是 API 文档的一部分。避免在 prose 中复制会和类型漂移的字段清单，除非该清单本身是 contract。

## README

README 面向第一次进入项目的人，覆盖当前可执行事实：

1. 项目用途和首期范围。
2. 快速开始。
3. 常用命令。
4. 架构入口和关键文档链接。
5. 开发、测试、发布或贡献的最小路径。

README 不应承载完整架构说明。需要背景时链接 ADR、`docs/navigation.md` 或对应主规范。

## CHANGELOG

CHANGELOG 记录已经交付且用户可见、集成方可见或运维可见的变化：

1. 新增功能。
2. 行为变更。
3. 修复。
4. 迁移说明。
5. 废弃和移除。

不要把内部重排、未发布实验或实现细节写成用户变化，除非它影响 public contract、性能、安全或操作方式。

## Agent-Facing Docs

AGENTS.md、rules files 和 project skills 只记录工作方式和上下文入口：

1. agent 应该从哪里开始读。
2. 哪些边界不能跨。
3. 哪些验证命令或安全规则必须执行。
4. 哪些规范拥有最终解释权。

避免复制主规范。长期规则要写成触发条件和验收标准，不堆叠临时提醒或坏示例。

## 判断规则

- “代码自文档化”只能覆盖 what，不能覆盖 why。
- “稳定后再写文档”通常会推迟设计反馈；public contract 需要先有可审阅文档。
- “没人读文档”不是理由；agent、reviewer、未来维护者都会依赖入口文档。
- “注释会过期”说明注释写在了错误层级；解释 why 的注释通常比解释 what 更稳定。
