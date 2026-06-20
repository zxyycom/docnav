# Docnav 文档同步

## 入口

在 Docnav 仓库内读取项目文档时，先进入 `docs/navigation.md` 的“如何阅读这些文档”。按任务角色读取主规范，不默认展开 OpenSpec、schema 或 examples。

## 同步矩阵

| 变化面 | 必查或必改材料 |
| --- | --- |
| protocol 字段、错误、pagination、continuation | `docs/protocol.md`、相关 schema、examples、tests |
| CLI readable/raw output | `docs/cli.md`、examples、CLI tests |
| adapter behavior、probe、outline、read、find、ref | `docs/adapter-contract.md`、`docs/refs.md`、format-specific docs、adapter tests |
| MCP tool mapping | `docs/cli.md`、`docs/protocol.md`、MCP bridge tests |
| schema field 或 validation | `docs/schemas/`、`docs/examples/`、schema validation tests |
| user-facing setup 或 commands | `README.md`、`docs/navigation.md`、relevant scripts or CI docs |
| architecture rationale | 长期架构决策写 ADR；change-local rationale 写 OpenSpec design；当前 contract 写主规范 |

## OpenSpec

- 处理 OpenSpec change、验收、归档或历史审计时，先运行 `openspec list --json`。
- OpenSpec 用于说明要改变什么行为、为什么这个 change 可接受，以及如何验收；不是 `docs/` 主规范的 owner。
- 长期架构 rationale 可以写 ADR 并链接 OpenSpec，但不要把 OpenSpec proposal、tasks、spec delta 或 acceptance 复制进 ADR。
- 主规范可以记录当前行为、目标方向、计划能力和已接受决策；状态语义和冲突归并规则由 `docs/navigation.md` 拥有。ADR 只解释为什么长期选择这条路。

## Schema 与 Examples

`docs/schemas/` 和 `docs/examples/` 是验证材料。只有当变更触及字段、示例、protocol output、test fixture 或 contract compatibility 时才读取或编辑。

同步完成的验收标准：

1. 主规范描述对应行为或目标，并按需要区分 Current、Target、Planned 或 Historical。
2. schema 能验证对应 raw protocol。
3. examples 展示真实输出或输入。
4. tests 覆盖变更面。
5. 文档没有把目标能力写成 Current 或已交付事实。
