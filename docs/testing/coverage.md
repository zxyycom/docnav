# 覆盖矩阵

本文供测试作者和 reviewer 判断改动是否触达最低覆盖面，只定义最低覆盖维度；不列具体测试用例，不定义稳定字段、错误码、命令语义或字段形状。产品语义以 [文档导航](../navigation.md#规则所有权) 指向的 owner 文档为准。

测试函数变更时，按 [测试用例维护](case-maintenance.md) 判断 case 归属；最终 case 条目记录在 [测试用例编号账本](cases.md)。

## 使用方式

1. 先按 [测试策略](../testing.md) 选择测试层级。
2. 再用本文确认改动触及哪些覆盖维度。
3. 新增证明目标时，按 [测试用例维护](case-maintenance.md) 判断登记、合并或拆分 case，并补源码 `@case` 标记。

## 覆盖维度

| 维度 | 最低覆盖要求 | 主要测试层 |
| --- | --- | --- |
| Shared output 编排 | 从 success/failure `ProtocolResponse` 覆盖 `ProtocolJson` 与 `Rendered(RenderStrategy)`；rendered path 覆盖 built-in/custom renderer、exact text、`RenderFailure` before stdout、no fallback 和独立 writer failure。 | Rust output/renderer tests |
| CLI output mapping 与 migration | 省略 output、显式 `readable-view`、`protocol-json` 和提前 document failure 各保留代表；已删除 `readable-json` 以 CLI/config 普通 invalid-value 等价类代表，不建立旧值矩阵。 | CLI smoke、Rust core/parser/config tests |
| Protocol/rendered isolation | `ProtocolJson` 不受 renderer availability/behavior 影响并继续符合原始协议 schema；built-in conformance 从同一 `ProtocolResponse` 验证最终 `readable-view` text。 | CLI smoke、protocol integration、readable conformance tests |
| 命令族 | 每个正式命令族至少覆盖一个成功路径、一个代表性失败或 help 边界；不为参数组合建立笛卡尔积。 | CLI smoke、Rust parser/config tests |
| 文档能力 | `outline`、`read`、`find`、`info` 覆盖 core CLI、static registry adapter dispatch 和 protocol/readable 输出中的代表路径。 | CLI smoke、Rust adapter/protocol tests |
| adapter inspection | descriptor metadata、static registry membership 和 `adapter list` 覆盖 static registry metadata、linked handler availability 和 adapter layer 可用性；manifest/probe-shaped JSON 只作为 schema/example contract material。 | CLI smoke、schema/docs validators、Rust core/adapter tests |
| adapter source boundary | 默认 adapter implementation source 是 core release static registry 中的 linked adapter libraries。 | Core CLI smoke、Rust core tests |
| ref 与分页 | 至少覆盖 `outline -> ref -> read`、`find -> ref -> read`、invalid/not-found ref、分页继续和终止。 | CLI smoke、Rust adapter tests |
| Success-only auto-read 编排 | 覆盖 CLI/project/user/built-in 来源优先级、省略来源时的 default-on dispatch、CLI/config disable compatibility、当前返回 ref eligibility、nested non-success 静默保留 base response、composed protocol/readable projection，以及单根 invocation event、显式 content capture 与默认不记录正文；按 owner 分层选择代表，不建立参数组合矩阵。 | Rust core/navigation/protocol/output tests、readable conformance、CLI smoke |
| 诊断模型与投影阶段 | 覆盖 CLI 输入错误、adapter selection explicit failure、missing adapter + invalid-looking option 时 selection diagnostic 优先、automatic discovery all-failed probe candidate list、selected adapter layer failure、selected-adapter typed-field option validation failure、ref error、primary `DiagnosticRecord` protocol/readable 投影、canonical details 和从属 details 语义的代表样本。 | CLI smoke、schema/docs validators、Rust diagnostics/output tests |
| Navigation input resolution 与 path context | 覆盖 explicit/project/user/built_in 来源合并、`--path` context、navigation-owned raw config source loading、operation-scoped registry/selected field-set parity、generated help 与 lexical/preflight facts、normalized candidate handoff、selected member resolution 与 unselected explicit failure、internal protocol request/typed handler handoff、adapter option declaration 和 config namespace、default config absence、invalid/shape failure、help 不读取配置和解析结果不回写原始 protocol JSON 的边界。 | Core CLI smoke、Rust parser/config/navigation/adapter tests |
| release package | 覆盖 core-only package manifest、文件集合、校验和、host/target 选择和 package 内 `docnav` 二进制 smoke；linked adapters 通过 core CLI 行为证明。 | release package scripts、package smoke |

## 层级选择

- CLI smoke：证明真实 core CLI 入口、stdout/stderr、exit code、strict failure/error 投影承载位置和 package 可执行性。
- Rust tests：证明 parser、ref、分页、decode stage、diagnostic record/code/details/投影 helper、shared output plans、renderer 和内部状态转换等自定义逻辑不变量。
- schema/docs validators：证明 protocol 字段形状、示例链路、schema 投影映射和文档化 fixture 与当前 owner 文档一致；`readable-view` 由 conformance text 验证，schema/example/fixture 不成为 code/details 规则来源。
- 测试用例维护：定义测试函数变更时的 case 归属、账本更新和 `@case` 标记维护流程。
- 测试用例编号账本：保存最终 case 条目、证明目标和源码 `@case` 标记映射，不替代测试实现或覆盖矩阵。

## 审查规则

1. 新测试应证明新的行为边界、责任层级或等价类；同一等价类的更多参数值优先下沉到较低层测试或现有 case 断言。
2. 外部入口只保留代表路径；字段全集、错误全集和参数全集由 owner 文档、schema、fixtures 或 Rust tests 证明。
3. 改动跨多个维度时，至少为每个受影响维度保留一个可执行证明；无需为所有维度做交叉乘积。
