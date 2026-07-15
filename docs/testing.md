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
| 单元 | parser、ref、默认值、分页、diagnostic record/projection helper 等可直接调用并观察的模块行为 |
| 集成 | `docnav` 配置优先级、static registry adapter 选择、adapter inspection、protocol request dispatch、输出模式和真实 CLI 通道 |
| 端到端 | 真实 CLI、release package、协议映射、精简输出和 continuation 链路 |

## 测试所有权

测试按观察边界划分所有权。同一行为只有在不同层能观察到不同 owner 结果时才跨层测试：Rust tests 直接观察模块/API 的输入输出，CLI smoke 观察真实进程入口的退出码、stdout、stderr 和文件效果。测试不证明内部实现路径、函数组织或背后逻辑。

测试首先是当前有效性证据：执行结果必须直接表明 owner 承诺的行为当前成立。发现未来变化只是自动化验证的自然结果，不是 case 的证明目标；不得用“能够防止某种回归”、实现采用了哪条内部路径，或调用方无法观察的背后逻辑扩大 `Proves:`。

历史回归只作为风险线索或代表性输入来源，不作为证明目标。无论是新增 case、拆分 case，还是把断言嵌入已有 case，新增断言都必须先能写出“owner 明确承诺的语义 -> 可观察结果”的证明目标，并能追溯到当前 owner 文档、schema、示例、primary failure projection 或覆盖矩阵。只有当明文契约要求校验缺失、拒绝、输出通道不污染、ref 不改变或其它否定性边界时，才测试该否定行为；否则使用现有覆盖、局部验证命令或代码审查证明本次改动。

维护测试和测试文档时遵循两条约束：owner 文档明确承诺的行为，自动化测试应断言调用方能够观察到的结果；owner 没有承诺、或当前层级无法可靠观察的行为，应缩小 case 的 `Proves:`，不得为了匹配账本文字增加防御性断言、测试专用探针或对实现细节的反向推断。

测试按语义类型和等价类选择代表，不按语法拼写、枚举字面量或参数组合穷举。“一个类型”表示具有独立解析、校验、状态转换、输出 shape 或失败投影的可观察行为；同一行为中的多个合法写法或同类非法值只保留一个代表。只有 owner 明确定义了不同语法分支，或不同输入产生不同可观察结果时，才分别测试。

如果自动化验证必须复制被测实现、引入只为测试存在的观测接口、依赖脆弱外部环境，或其长期维护成本明显高于语义漂移风险，可以不新增自动化测试。此时在 owner 文档的验证说明或变更审查记录中写明 `Manual CR:`、审查对象和判定条件；不得用空断言、恒真断言或名义上的 implemented case 代替人工审查。

### CLI smoke

CLI smoke 从发布给用户的可执行入口验证外部契约。覆盖范围按以下维度评估：

- 所有命令族的代表性路径，以及关键成功和失败场景。
- `readable-view`、`readable-json` 和 `protocol-json` 三种输出模式。
- 退出码、`stdout`、`stderr` 及其相互约束。
- strict failure/error 投影的承载位置、stdout/stderr 边界和 schema 校验。
- static registry adapter source boundary、`adapter list` metadata inspection 和 adapter command surface。
- protocol raw result facts 到 readable `display`、成本摘要和 info 摘要的跨层映射。
- 分页、continuation 和终止行为。
- Markdown document head 的真实 CLI 链路：fixture 同时包含 frontmatter、普通前导正文和 heading，并在 `protocol-json` 中验证 raw entry facts 不包含 readable-only `display`，在 `readable-json` / `readable-view` 中验证 display 和 content block 为输出层派生。
- core 和 release package 的真实 CLI 链路。

配置场景使用 `test/smoke/core/fixtures/configs/` 下长期保留、按语义命名的 JSON fixture。CLI smoke 可以把 fixture 安装为临时 project 的 `.docnav/docnav.json` 来验证默认 project context，也可以把只读 fixture 直接通过 `--project-config` / `--user-config` 传给真实 CLI。长期 public config CLI 只保留只读 `docnav config inspect`：config command proof 必须覆盖 selected project/user source status、explicit missing / invalid JSON / top-level non-object / not-file config source status、source-attributed validation diagnostics、当前可解析参数事实展示、一个有写入风险的旧 config editor subcommand 经 parser/error boundary 拒绝且不修改文件，以及 inspect 不修改 config file。真实 CLI smoke 保留 selected source、参数事实、source-attributed diagnostic、legacy rejection 和 read-only 边界的代表；其余 load state 可以由直接执行 config command 并校验 serialized output 的 Rust tests 证明，不在进程层重复。`get|set|unset|list` 属于同一 removed-subcommand 等价类，不建立名称矩阵。Explicit unreadable source loading 归属 lower-layer config loading / parameter-resolution 或 navigation-owned raw config source tests，不要求 command-level smoke 复测。需要证明 direct edit/read 的场景应通过 fixture 或临时 config file 内容准备，不通过 config editor command 写入。

每个契约维度至少保留一个代表性用例。同一校验规则下的多个同类非法值视为一个等价类，只选择能证明外部行为的用例。覆盖完整性由契约维度判断，不以代码覆盖率或参数组合数量衡量。

### Rust tests

Rust tests 负责无需启动真实 CLI 即可直接观察的 owner 行为。每个用例应明确证明一种输入输出分支、状态转换、数据约束或失败投影；实现内部如何得到该结果不属于证明内容。例如：

- 参数 token 消费边界、config path flag 支持命令集合、missing value 和 strict unmapped-input 规则。
- core 配置 source descriptor/path 解析与 handoff，包括 explicit exact file path、user config fallback、project context fallback、source level、resolved path 和 path origin。
- core config inspect command 的 read-only source inspection 边界，包括旧 config 子命令移除、selected source loading status、source summary、validation diagnostic projection、参数事实展示和不构造 document operation request。
- parameter aggregation projection parity：CLI/input projection 与 config-source projection 复用同一 owner-provided facts，core config command 不重新定义 output enum、positive integer、adapter option value kind/range/default 或 outline selector 语义。
- navigation-owned raw config source 读取、default missing absence、explicit missing/unreadable/invalid/non-object failure、navigation input resolution 来源合并、通用字段声明与 selected adapter declarations 注册、adapter option handoff、adapter-id native option namespace 和 config failure 边界。
- adapter-id option namespace：`options.<adapter-id>.<option-key>` 在不同 adapter id 下保持 deterministic，裸 `options.<option-key>` 按普通 unknown/invalid config path 处理，navigation 只 forward selected adapter namespace 的 declared operation option。
- `outline.mode_rules[]` 和 `outline.auto_full_read.thresholds[]` 的 owner-specific config validation parity，包括 source path、unknown item key、required member、typed value、matcher/threshold diagnostics 和 navigation resolution diagnostics。只有 parity 不足时才新增 typed-fields compound helper tests。
- operation 参数所有权。
- Markdown 解析、ref 生成和定位。
- Markdown document head 范围与 eligibility：满足条件时始终暴露 `HEAD:leading`，空或纯空白 document head 不暴露 entry，无可见 heading 时保留 `doc:full` fallback。
- Markdown document head read/find：frontmatter delimiter 和普通前导正文原文保留，`HEAD:leading` 返回 `text/markdown`，find 命中 document head 后可用返回 ref 继续 read，Unicode 分页不拆分字符。
- Unicode 字符预算、分页和终止规则。
- protocol、manifest 和 probe decode wrapper 只按实际可达行为选择 schema、typed result 或 semantic failure 代表；无法从具体 wrapper 构造的 generic deserialize fallback 不使用假 schema gate 建立测试。
- DiagnosticCode details、primary `DiagnosticRecord` 投影、从属 details 语义、core CLI argv strictness、document output orchestration、低层 JSON writer 和 paging helper 的可观察行为边界。
- protocol/readable 行为隔离：config inspect 和 config-source validation 的当前行为不改变 document operation `protocol-json`、`readable-json`、`readable-view` stdout 或 linked adapter handler payload。

Manual CR: 修改 `DiagnosticCode -> details rule` 的完整字段表时，reviewer 同步核对 typed details payload、`code/details.rs` 和 protocol schema/example。自动化测试只为每种 details field type 保留一个验证代表，并验证具有独立 public projection 的具体 code；不维护一份由被测规则复制出来的全量 expected table。

Manual CR: 修改 protocol、manifest 或 probe 的 schema 与 Rust typed shape 映射时，reviewer 对照 public schema、对应 Rust type 和具体 decode wrapper；若 schema-valid 但 typed-invalid 输入无法从该 wrapper 的真实 surface 构造，不用 generic type 或恒真 gate 伪造自动化分支。

以下行为由 CLI smoke 验证外部契约，无需在 Rust 中建立重复矩阵：

- `clap` 自带的解析行为，例如简单缺少必填参数。
- 无自定义分支的字段透传、转换和输出模式枚举选择。
- 同一校验规则下的多个等价非法值。
- `readable-view` block framing、stdout/stderr 分流和用户可观察输出边界。

### 代码组织

- Rust 白盒测试放在对应 `tests.rs` 子模块，主实现文件只声明测试模块。
- 测试通过模块可见性访问私有实现，生产 API 的可见性保持不变。
- 单个测试只证明一个可观察行为类型。
- 参数解析测试保持少量高价值用例；新增用例必须覆盖新的 strict input 规则、token 消费边界或 operation 参数所有权不变量。
- 跨层测试必须分别断言各层独有的可观察结果，不重复相同的参数组合矩阵。

## 脚本与工具依赖

验证脚本和按需工具依赖的运行方式由 [工程工具链](tooling.md) 拥有。本节只定义测试验证边界：

`scripts/` 下工程脚本的自动化测试只证明重要 public semantics 当前有效，不证明“脚本整体没有问题”，也不以防止未来语义漂移为证明目标。它们只需从 public source entrypoint、稳定输入输出、错误映射或调度结果中，为每种重要语义类型保留一个代表；不要求按 helper、分支、命令参数或第三方工具输出建立完整矩阵。简单编排脚本若没有独立语义，可以只依赖 typecheck、lint、实际验证命令和 Manual CR。

- 包依赖不要求预先全局安装；TypeScript 脚本运行时要求环境提供 Bun，具体前置条件由 [工程工具链](tooling.md) 维护。
- `bun run typecheck:scripts` 证明脚本模块 contract 和边界类型一致，不替代真实 CLI、schema 或 smoke 验证。
- `bun run lint:scripts` 证明脚本源码没有未使用变量/函数、显式 `any` 和常见静态质量问题。
- 共享脚本子仓库的 private manifest 提供局部 `typecheck`、`lint` 和 focused `test` 入口；full profile 运行其中证明独立 public semantics 的 focused tests。private manifest 是本地聚焦入口，不要求为每个 package 或测试函数建立额外账本项。

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

required profile 是快速、确定性的必需验证集合，用于日常开发中缩短反馈周期，适合改文档、修脚本或调验证逻辑时先跑。它包含 quick quality check；该检查跳过 baseline comparison 和 jscpd duplicate detection，因此出现 warning 时会提示当前不是全量质检。full profile 复用 required profile 中的非质量必需检查，使用 full quality check 替代 quick quality check，并追加质量观测内部测试、CLI smoke、Rust 全量测试、cargo clippy 和 OpenSpec 严格校验。

full profile 会验证质量观测链路本身：工具封装测试、扫描执行、配置读取和输出结构必须通过。Lizard、scc 和 jscpd 的观测结果进入快照、报告和 warning records；单独质量扫描存在 warning records 时继续显示 `warning`。workspace verifier 的 full profile 使用 verifier 输出：只有未带 `acceptedReason` 的 warning records 会把 workspace verifier 状态标记为 `warning`，带 `acceptedReason` 的 warning 仍写入质量 artifact 和报告，并在对应 warning 旁展示原因。

required profile 包含 `typecheck:scripts`、`lint:scripts` 和 quick quality check，分别验证 `.ts` 脚本类型 contract、静态质量规则和轻量质量观测状态。

workspace verifier 的终端输出用于快速判断当前验证状态：默认展示每个 report 的 completion line 和最终 summary。运行期输出按职责分离：

- 完整子命令 stdout/stderr 写入 `.log/verify/workspace/latest.log`。
- full profile 中的 core smoke 审计日志写入 `.log/smoke/core/latest.log`。
- 验证运行中间状态写入 `.cache/docnav/verify/`。
- 一次性 smoke 临时工作区写入 `.tmp/docnav/smoke/`。

workspace verifier 会为 quality check 设置 `DOCNAV_QUALITY_TIMINGS=1`；耗时分解保留在 workspace verifier 日志中，终端仍只展示 completion line、warning 摘要或失败诊断。

各 check 的终端可见行由验证脚本中的输出白名单/黑名单维护：passed 只需要 completion line，warning 保留可行动摘要和报告路径，failed 保留失败诊断。

Manual CR: 修改 workspace verifier 的 check definitions、命令参数、dependencies、mutex 或 Codex environment setup 时，reviewer 运行 required/full profile，并对照 completion lines 与 `.log/verify/workspace/latest.log` 核对实际命令和执行关系；focused tests 只证明 profile 选择、输出过滤、状态映射和 report 计数，不复制完整 check 配置表。

开发期快捷入口：

| 命令 | 用途 |
| --- | --- |
| `bun run verify:docnav-workspace:required` | 快速验证，只跑必需检查 |
| `bun run quality:check` | 快速质量检查，生成 quick profile 报告 |
| `bun run quality:full-check` | 全量质量检查，包含 baseline comparison |
| `bun run smoke:docnav` | 对当前开发构建运行 core CLI smoke |
| `bun --silent run dnm <args>` | 运行当前开发版 `docnav`，只保留命令结果和失败诊断 |

局部改动仍可先运行范围更小的命令或 required profile；跨 Rust、文档、OpenSpec、schema、示例或输出层边界的交付，最终应运行 `bun run verify:docnav-workspace`。具体检查项和输出过滤规则由验证脚本维护，本节只定义 profile 用途和交付要求。

## 一致性审计

交付前检查：

1. 新增、删除或修改测试能追溯到 [文档导航](navigation.md#规则所有权) 指向的 owner 文档。
2. 测试函数变更已按 [测试用例维护](testing/case-maintenance.md) 判断证明目标、case 归属和账本更新范围。
3. 测试文档只记录覆盖目标和验收边界，不重新定义稳定字段、错误码、DiagnosticCode details 规则或命令语义。
4. schema、示例和 fixture 只校验 protocol raw shape、readable 输出投影和二者的 documented mapping，不成为新的业务语义或 code/details 规则来源。
5. OpenSpec change 只作为变更依据、验收和审计历史，不作为日常实现主入口。
6. 当测试暴露规范缺口时，先更新 owner 文档，再同步 schema、示例、实现和验证脚本。
7. 涉及共享 helper 的改动必须覆盖可观察外部行为：core CLI strict failure placement、protocol-json stdout purity、primary readable/protocol error projection、protocol raw facts 到 readable display/cost/info projection、static registry adapter inspection、core config descriptor/path handoff、selected config file target behavior、navigation-owned raw config source loading 与 navigation input resolution 边界、Markdown pagination mechanics 和 schema/decode/semantic invalid paths。
