# 覆盖矩阵

本文供测试作者和 reviewer 判断改动是否触达最低覆盖面。它不列具体测试用例，不定义稳定字段、错误码、命令语义或字段形状；产品语义以 [文档导航](../navigation.md#规则所有权) 指向的 owner 文档为准，具体证明目标和源码 `@case` 标记见 [测试用例编号账本](cases.md)。

## 使用方式

1. 先按 [测试策略](../testing.md) 选择测试层级。
2. 再用本文确认改动触及哪些覆盖维度。
3. 若新增了新的证明目标，在 [测试用例编号账本](cases.md) 登记编号并补源码 `@case` 标记。

## 覆盖维度

| 维度 | 最低覆盖要求 | 主要测试层 |
| --- | --- | --- |
| 输出层 | `readable-view`、`readable-json` 和 `protocol-json` 至少各有代表性外部入口断言；schema/readable 映射由验证脚本覆盖。 | JavaScript smoke、schema/docs validators、Rust renderer tests |
| 命令族 | 每个正式命令族至少覆盖一个成功路径、一个代表性失败或 help 边界；不为参数组合建立笛卡尔积。 | JavaScript smoke、Rust parser/config tests |
| 文档能力 | `outline`、`read`、`find`、`info` 覆盖 core CLI、直接 adapter CLI 和 invoke 链路中的代表路径。 | JavaScript smoke、Rust adapter/protocol tests |
| adapter 机器能力 | `manifest`、`probe` 和 `invoke` 覆盖 schema-valid stdout、stdin decode failure 和语义失败阶段。 | JavaScript smoke、schema/docs validators、Rust protocol/SDK tests |
| adapter 管理 | `adapter list/install/update/remove` 覆盖正式流程、manifest 校验、fingerprint 边界和错误映射。 | Core CLI smoke、Rust core tests |
| ref 与分页 | 至少覆盖 `outline -> ref -> read`、`find -> ref -> read`、invalid/not-found ref、分页继续和终止。 | JavaScript smoke、Rust adapter tests |
| 错误与 warning 阶段 | 覆盖 CLI 输入错误、adapter selection warning、candidate failure、selected invoke failure、ref error 和 warning placement 的代表样本。 | JavaScript smoke、Rust diagnostics/output tests |
| 配置与 path context | 覆盖 user/project/default 优先级、`--path` context、非法配置值和配置不改变协议字段的边界。 | Core CLI smoke、Rust config tests |
| MCP bridge | 覆盖 tool call 到 `docnav` CLI 的映射、TextContent/structuredContent 分层和 readable schema 校验。 | MCP bridge tests、schema/docs validators |
| release package | 覆盖 manifest、文件集合、校验和、host/target 选择和 package 内二进制 smoke。 | release package scripts、package smoke |

## 层级选择

- JavaScript smoke：证明真实进程入口、跨二进制链路、stdout/stderr、exit code、warning placement 和 package 可执行性。
- Rust tests：证明 parser、ref、分页、decode stage、helper、renderer 和内部状态转换等自定义逻辑不变量。
- schema/docs validators：证明字段形状、示例链路、schema 映射和文档化 fixture 没有漂移。
- 测试用例编号账本：只维护审计编号、证明目标和源码 `@case` 标记映射，不替代测试实现或覆盖矩阵。

## 审查规则

1. 新测试应证明新的行为边界、责任层级或等价类；同一等价类的更多参数值优先下沉到较低层测试或现有 case 断言。
2. 外部入口只保留代表路径；字段全集、错误全集和参数全集由 owner 文档、schema、fixtures 或 Rust tests 证明。
3. 改动跨多个维度时，至少为每个受影响维度保留一个可执行证明；无需为所有维度做交叉乘积。
