# 测试策略

本文定义 Docnav 自动化测试的层级、所有权、统一验证入口和一致性审计规则。以下子文档分别维护测试用例流程、最终账本、覆盖矩阵和发布包预验收：

- [测试用例维护](testing/case-maintenance.md)：测试函数变更时的 case 归属、`@case` 标记和账本更新流程。
- [测试用例编号账本](testing/cases.md)：最终 case 条目、证明目标和源码 `@case` 标记映射。
- [覆盖矩阵](testing/coverage.md)：跨入口、命令族和文档操作的最低覆盖目标。
- [发布包验证](testing/release.md)：release package 的本地预验收和 CI/CD 验证边界。

稳定字段、错误码、命令语义、adapter 行为和字段形状以 [文档导航](navigation.md#规则所有权) 指向的 owner 文档为准；测试文档只记录覆盖目标和验收边界。

## 测试层级

| 层级 | 核心目标 |
| --- | --- |
| schema | 原始协议、manifest、probe 和各 operation readable 输出分别通过独立 schema；readable schema 用于示例和工具输出校验，不作为完整机器协议 |
| 单元 | parser、ref、默认值、分页、错误通道记录/投影和其它自定义逻辑不变量 |
| 集成 | `docnav` 配置优先级、static registry adapter 选择、adapter inspection、protocol request dispatch、输出模式和真实 CLI 通道 |
| 端到端 | 真实 CLI、release package、协议映射、精简输出和 continuation 链路 |

## 测试所有权

测试按“用户可观察契约”和“自定义逻辑不变量”划分所有权。同一行为只有在证明不同责任时才跨层测试：Rust tests 证明内部不变量，CLI smoke 证明真实进程入口的外部契约。

历史回归只作为风险线索或代表性输入来源，不作为证明目标。无论是新增 case、拆分 case，还是把断言嵌入已有 case，新增断言都必须先能写出“owner 明确承诺的语义 -> 可观察结果”的证明目标，并能追溯到当前 owner 文档、schema、示例、错误通道投影或覆盖矩阵。只有当明文契约要求校验缺失、拒绝、输出通道不污染、ref 不改变或其它否定性边界时，才测试该否定行为；否则使用现有覆盖、局部验证命令或代码审查证明本次改动。

### CLI smoke

CLI smoke 从发布给用户的可执行入口验证外部契约。覆盖范围按以下维度评估：

- 所有命令族的代表性路径，以及关键成功和失败场景。
- `readable-view`、`readable-json` 和 `protocol-json` 三种输出模式。
- 退出码、`stdout`、`stderr` 及其相互约束。
- strict failure/error 投影的承载位置、stdout/stderr 边界和 schema 校验。
- static registry adapter source boundary、`adapter list` metadata inspection 和 adapter command surface。
- protocol raw result facts 到 readable `display`、成本摘要和 info 摘要的跨层映射。
- 分页、continuation 和终止行为。
- core 和 release package 的真实 CLI 链路。

每个契约维度至少保留一个代表性用例。同一校验规则下的多个同类非法值视为一个等价类，只选择能证明外部行为的用例。覆盖完整性由契约维度判断，不以代码覆盖率或参数组合数量衡量。

### Rust tests

Rust tests 负责具有独立出错空间的自定义逻辑。每个用例应明确证明一个分支、状态转换、算法边界或数据不变量，例如：

- 参数 token 消费边界和 strict unmapped-input 规则。
- core 配置 source descriptor/path 解析与 handoff、navigation-owned raw config source 读取、navigation input resolution 来源合并、selected adapter typed-field 参数声明、源码级 native option registry/generic merge、adapter option handoff 和 config failure 边界。
- operation 参数所有权。
- Markdown 解析、ref 生成和定位。
- Unicode 字符预算、分页和终止规则。
- protocol、manifest 和 probe decode pipeline 的 schema invalid、typed deserialize invalid 和 semantic invalid 边界。
- diagnostics stack/id/mark/order、DiagnosticCode details/primary `DiagnosticRecord` 投影、core CLI argv strictness、document output orchestration、低层 JSON writer 和 paging helper 的可观察行为边界。

以下行为由 CLI smoke 验证外部契约，无需在 Rust 中建立重复矩阵：

- `clap` 自带的解析行为，例如简单缺少必填参数。
- 无自定义分支的字段透传、转换和输出模式枚举选择。
- 同一校验规则下的多个等价非法值。
- `readable-view` block framing、stdout/stderr 分流和用户可观察输出边界。

### 代码组织

- Rust 白盒测试放在对应 `tests.rs` 子模块，主实现文件只声明测试模块。
- 测试通过模块可见性访问私有实现，生产 API 的可见性保持不变。
- 单个测试只证明一个自定义不变量。
- 参数解析测试保持少量高价值用例；新增用例必须覆盖新的 strict input 规则、token 消费边界或 operation 参数所有权不变量。
- 跨层测试必须分别断言内部不变量和外部 CLI 契约，不重复相同的参数组合矩阵。

## 脚本与工具依赖

验证脚本和按需工具依赖的运行方式由 [工程工具链](tooling.md) 拥有。本节只定义测试验证边界：

- 包依赖不要求预先全局安装；TypeScript 脚本运行时要求环境提供 Bun，具体前置条件由 [工程工具链](tooling.md) 维护。
- `bun run typecheck:scripts` 证明脚本模块 contract 和边界类型一致，不替代真实 CLI、schema 或 smoke 验证。
- `bun run lint:scripts` 证明脚本源码没有未使用变量/函数、显式 `any` 和常见静态质量问题。

## 统一验证入口

常规交付前使用 Docnav workspace 综合验证入口：

```bash
bun run verify:docnav-workspace
```

该入口默认运行 full profile，是常规交付前的完整验证入口。

日常开发可先跑 required profile：

```bash
bun run verify:docnav-workspace:required
```

required profile 是快速、确定性的必需验证集合，用于日常开发中缩短反馈周期，适合改文档、修脚本或调验证逻辑时先跑。它包含 quick quality check；该检查跳过 baseline comparison 和 PMD CPD duplicate detection，因此出现 warning 时会提示当前不是全量质检。full profile 复用 required profile 中的非质量必需检查，使用 full quality check 替代 quick quality check，并追加质量观测内部测试、CLI smoke、Rust 全量测试、cargo clippy 和 OpenSpec 严格校验。

full profile 会验证质量观测链路本身：工具封装测试、扫描执行、配置读取和输出结构必须通过。Lizard、scc 和 PMD CPD 的观测结果进入快照、报告和 warning records；单独质量扫描存在 warning records 时继续显示 `warning`。workspace verifier 的 full profile 使用 verifier 输出：只有未带 `acceptedReason` 的 warning records 会把 workspace verifier 状态标记为 `warning`，带 `acceptedReason` 的 warning 仍写入质量 artifact 和报告，并在对应 warning 旁展示原因。

required profile 包含 `typecheck:scripts`、`lint:scripts` 和 quick quality check，分别验证 `.ts` 脚本类型 contract、静态质量规则和轻量质量观测状态。

workspace verifier 的终端输出用于快速判断当前验证状态：默认展示每个 report 的 completion line 和最终 summary。运行期输出按职责分离：

- 完整子命令 stdout/stderr 写入 `.log/verify/workspace/latest.log`。
- full profile 中的 core smoke 审计日志写入 `.log/smoke/core/latest.log`。
- 验证运行中间状态写入 `.cache/docnav/verify/`。
- 一次性 smoke 临时工作区写入 `.tmp/docnav/smoke/`。

各 check 的终端可见行由验证脚本中的输出白名单/黑名单维护：passed 只需要 completion line，warning 保留可行动摘要和报告路径，failed 保留失败诊断。

开发期快捷入口：

| 命令 | 用途 |
| --- | --- |
| `bun run verify:docnav-workspace:required` | 快速验证，只跑必需检查 |
| `bun run verify:docnav-workspace:full` | 完整验证，显式运行 full profile |
| `bun run quality:check` | 快速质量检查，生成 quick profile 报告 |
| `bun run quality:full-check` | 全量质量检查，包含 baseline comparison |
| `bun run smoke:docnav` | 对当前开发构建运行 core CLI smoke |
| `bun run cli:dev -- <args>` | 构建并运行当前开发版 `docnav` |
| `bun --silent run dnm <args>` | 运行当前开发版 `docnav`，只保留命令结果和失败诊断 |

局部改动仍可先运行范围更小的命令或 required profile；跨 Rust、文档、OpenSpec、schema、示例或输出层边界的交付，最终应运行 `bun run verify:docnav-workspace` 或 `bun run verify:docnav-workspace:full`。具体检查项和输出过滤规则由验证脚本维护，本节只定义 profile 用途和交付要求。

## 一致性审计

交付前检查：

1. 新增、删除或修改测试能追溯到 [文档导航](navigation.md#规则所有权) 指向的 owner 文档。
2. 测试函数变更已按 [测试用例维护](testing/case-maintenance.md) 判断证明目标、case 归属和账本更新范围。
3. 测试文档只记录覆盖目标和验收边界，不重新定义稳定字段、错误码、DiagnosticCode details 规则或命令语义。
4. schema、示例和 fixture 只校验 protocol raw shape、readable 输出投影和二者的 documented mapping，不成为新的业务语义或 code/details 规则来源。
5. OpenSpec change 只作为变更依据、验收和审计历史，不作为日常实现主入口。
6. 当测试暴露规范缺口时，先更新 owner 文档，再同步 schema、示例、实现和验证脚本。
7. 涉及共享 helper 的改动必须覆盖可观察外部行为：core CLI strict failure placement、protocol-json stdout purity、primary readable/protocol error projection、protocol raw facts 到 readable display/cost/info projection、static registry adapter inspection、core config descriptor/path handoff、navigation-owned raw config source loading 与 navigation input resolution 边界、Markdown pagination mechanics 和 schema/decode/semantic invalid paths。
