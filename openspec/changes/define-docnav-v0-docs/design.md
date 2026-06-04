## Context

Docnav 尚无实现代码，当前可用输入只有 `AGENTS.md` 中的项目原则和 OpenSpec 配置。首个版本选择先定义文档契约，使后续协议、适配器和 MCP 网关可以围绕同一组可审计要求独立实现。

`D:\project\skills\MarkdownNavigator` 提供了可运行的 Markdown CLI 参考。它已经验证了 heading 解析、章节范围、重复标题、深层标题、frontmatter、代码围栏和文件编码等行为，但其自由文本 selector、面向人类的错误输出和命令专用输出不能直接作为跨格式协议。

受影响的未来边界包括：

```text
AI Client <-> docnav-mcp stdio -> adapter invoke -> result
```

文档必须同时约束 `docnav-protocol`、`docnav-adapter-sdk`、`docnav-markdown`、`docnav-mcp` 和独立适配器 CLI，但本变更不实现这些制品。

## Goals / Non-Goals

**Goals:**

- 建立一套有明确所有权、受控术语和相互引用关系的 Docnav v0 文档。
- 通过完整 JSON 示例定义 `outline -> selector -> read` 的进程边界和协议行为。
- 明确 selector、错误、歧义处理、适配器识别和输出通道的稳定契约。
- 将 MarkdownNavigator 的已验证行为整理为可追踪的参考基线和迁移决策。
- 让后续实现变更能够直接从文档派生 schema、单元测试和端到端测试。

**Non-Goals:**

- 不创建 Rust workspace、crate、可执行文件或测试代码。
- 不实现 Markdown 或其他格式解析。
- 不完成安装、更新、移除、doctor 和 config 的内部实现设计。
- 不承诺 v0 之后所有协议字段永久不变；兼容性规则必须在协议文档中明确。

## Decisions

### 1. 文档按契约所有权拆分

项目文档采用以下最小集合：

| 文档 | 主要所有权 |
| --- | --- |
| `README.md` | 产品定位、核心流程、入口索引 |
| `docs/architecture.md` | 制品职责、进程边界、调用链 |
| `docs/protocol.md` | 版本、请求、响应、错误、能力声明 |
| `docs/selectors.md` | selector 生成、传递、唯一性和失效语义 |
| `docs/adapter-contract.md` | 适配器命令、`invoke`、manifest、probe、退出码 |
| `docs/cli.md` | 网关管理 CLI、适配器普通 CLI、输出通道 |
| `docs/testing.md` | 契约、兼容性、适配器和端到端验收矩阵 |
| `docs/references/markdown-navigator.md` | 参考行为、差异和迁移决策 |

选择按所有权拆分，而不是按可执行文件拆分，是为了减少协议规则在多个文档中重复并发生漂移。

### 2. 稳定机器契约与用户文案分离

`docs/protocol.md` 和 `docs/selectors.md` 中的字段名、枚举值、错误码和 JSON 示例属于机器契约；中文 guidance、usage 和错误建议属于可定制用户文案。文档必须明确两者的兼容性要求，避免将文案作为调用方解析依据。

### 3. selector 由 outline 生成并由 read 原样消费

文档必须定义 selector 为共享协议中的结构化值。调用方从 `outline` 结果取得 selector，并在 `read` 请求中原样传回；`docnav-mcp` 只校验共享 schema，不解释格式特有定位信息。

selector 必须能够在独立的适配器进程调用之间使用，且一次 `read` 不得通过“最近位置”静默选择多个候选。无法唯一定位、文档已变化或 selector 不再有效时，适配器返回结构化错误。

这一决策保留 MarkdownNavigator 的 `headings -> section` 工作流，但不继承自由文本路径和 `--line` 最近匹配作为协议 selector。

### 4. 文档示例作为后续测试向量

每个稳定请求、响应和错误形态至少提供一个完整 JSON 示例。端到端示例从 Markdown 输入开始，依次展示 MCP tool 请求、网关到适配器的 `invoke` 请求、适配器响应和最终 MCP 响应。

后续实现应将这些示例转为可执行 fixtures 或 schema 测试，避免文档示例与实现漂移。

### 5. MarkdownNavigator 只作为行为参考，不作为兼容目标

以下行为进入 Markdown 参考基线：

- 使用成熟 Markdown parser 识别 heading，并忽略代码围栏中的伪 heading。
- heading 章节从该 heading 开始，到下一个同级或更高级 heading 之前结束。
- 支持跳级和深层 heading，忽略空 heading。
- YAML frontmatter 不产生 heading。
- 重复标题和重复完整路径必须显式处理。
- 记录行数、范围大小、编码和截断等现有能力的迁移决策。

以下行为明确需要重做：

- `columns` 与位置数组组成的 headings 输出。
- 自由文本 `path`、`heading` 和近似 `line` 选择器。
- 只通过 stderr 中文文案表达错误。
- 对未知参数发出警告后继续执行。
- `section` 成功时只输出无 envelope 的原始文本。

### 6. 文档审计以中文为主

所有面向审计者的说明、设计理由、requirements、tasks 和用户可见示例解释使用中文。命令、路径、JSON 字段、错误码、capability 名称和协议枚举保留英文，以确保机器可读性和跨语言实现一致性。

## Risks / Trade-offs

- [先写文档可能形成未经实现验证的契约] → 每个关键契约必须包含示例、边界案例和待实现测试映射；首个实现变更允许通过新的 OpenSpec change 修正文档。
- [文档拆分后可能出现重复和漂移] → 为每类规则指定唯一主文档，其他文档只链接和展示使用方式。
- [opaque selector 可能降低可调试性] → outline 节点同时提供稳定展示信息和诊断元数据，但调用方不得自行构造格式特有定位值。
- [MarkdownNavigator 行为可能被误认为兼容承诺] → 单独维护参考文档，并为每项行为标注“保留、调整、推迟或移除”。
- [协议示例过早固定字段] → 在文档任务中集中审查 v0 最小字段，仅固定端到端链路需要的字段。

## Migration Plan

1. 完成并审计 v0 文档集合。
2. 以文档为输入提出协议与 Markdown 纵向链路实现变更。
3. 将文档 JSON 示例转换为 schema fixtures 和端到端测试。
4. 实现验证发现契约缺陷时，通过独立 OpenSpec change 修改文档并说明兼容影响。

本变更仅新增文档，无运行时回滚要求；撤销时可删除新增文档和对应 OpenSpec requirements。

## Open Questions

- selector 的内部载荷采用可读结构还是 opaque 编码，需要在 `docs/selectors.md` 中结合可调试性和跨版本兼容性最终确定。
- v0 是否要求非 UTF-8 文档支持，需要在 `docs/protocol.md` 和 Markdown 参考迁移表中明确。
- `document_find` 和 `document_info` 是否需要与首条 `outline -> read` 示例同时完整定义，需要在文档编写阶段收敛。
