# Smoke Case 清单

本文记录 JavaScript smoke 的 case inventory 和维护规则。[测试策略](../testing.md) 负责测试层级、验证入口和验收边界；本文只说明 smoke case 如何按外部链路类型组织，避免把脚本目录变成新的规则来源。

## 组织规则

1. Smoke 只验证真实进程入口的用户可观察契约，不覆盖细粒度 parser、renderer、schema 或 adapter 内部语义。
2. 用例按外部链路类型管理，不按 operation、output mode、非法参数或 fixture 做笛卡尔积扩展。
3. 同一类型只保留一条能证明边界行为的代表用例；同类非法值、同阶段错误和同输出层组合优先下沉到 Rust tests、schema validators 或现有 case 断言。
4. 连续链路可以在一个 case 内串行执行多个 CLI 命令；只有独立链路才拆成独立 task 调度。
5. 稳定字段、错误码、命令语义和 schema shape 以 [文档导航](../navigation.md#规则所有权) 指向的 owner 文档为准。

## Core CLI

Core CLI smoke 位于 `test/smoke/core/cases/`，只覆盖核心 `docnav` 真实进程入口和跨 adapter 链路。

| Case ID | 文件 | 覆盖目标 |
| --- | --- | --- |
| `CORE-LINK-001` | [real-markdown.ts](../../test/smoke/core/cases/real-markdown.ts) | 真实 `docnav` + `docnav-markdown` 的 `outline -> ref -> read`、`find -> ref -> read` 与 `info` 串行链路；证明 core 原样传递 adapter ref，并保持 readable-json 输出分层。 |
| `CORE-REF-001` | [real-markdown.ts](../../test/smoke/core/cases/real-markdown.ts) | 真实 Markdown adapter 返回的 ref 错误映射为 `protocol-json` failure；证明 adapter-owned ref 错误能穿过 core。 |
| `CORE-OUTPUT-001` | [outputs.ts](../../test/smoke/core/cases/outputs.ts) | `readable-json`、显式/默认 `readable-view`、`protocol-json` 和 warning 承载边界；不枚举所有 operation。 |
| `CORE-ARGS-001` | [cli-args.ts](../../test/smoke/core/cases/cli-args.ts) | 当前 operation 实际使用参数缺失时返回 protocol failure；其它同类非法值属于 parser/config 单测。 |
| `CORE-CONFIG-001` | [config-management.ts](../../test/smoke/core/cases/config-management.ts) | project/user config 优先级、非法 output 值、`config list --path` 的 adapter/path context；不枚举全部 config get/set/unset 组合。 |
| `CORE-SELECT-001` | [adapter-selection.ts](../../test/smoke/core/cases/adapter-selection.ts) | 显式预选 adapter manifest 失败后继续 registry fallback，并在 readable-json 中输出 candidate warning。 |
| `CORE-FAIL-001` | [failures.ts](../../test/smoke/core/cases/failures.ts) | candidate 阶段 adapter 进程失败被记录为 `FORMAT_UNKNOWN` candidate evidence；不逐项枚举 manifest/probe/contract/process 组合。 |
| `CORE-INVOKE-001` | [failures.ts](../../test/smoke/core/cases/failures.ts) | invoke 阶段 adapter 进程失败映射为 `ADAPTER_INVOKE_FAILED`；证明 selected adapter invoke failure 与 candidate failure 属于不同错误阶段。 |
| `CORE-TOOLS-001` | [config-management.ts](../../test/smoke/core/cases/config-management.ts) | `init`、`version`、`doctor` 和 document help 的可用性；保留非 document operation 的主要入口 smoke。 |

## Markdown CLI

Markdown CLI smoke 位于 `test/smoke/markdown/cases/`，只覆盖 `docnav-markdown` 直接 CLI 的真实进程边界。

| Case ID | 文件 | 覆盖目标 |
| --- | --- | --- |
| `MD-LINK-001` | [outputs.ts](../../test/smoke/markdown/cases/outputs.ts) | `outline -> ref -> read`、`find -> ref -> read` 和 `info` 的 readable-json 串行链路；证明直接 CLI 主链路和 ref 生成/读取。 |
| `MD-OUTPUT-001` | [outputs.ts](../../test/smoke/markdown/cases/outputs.ts) | `readable-json`、显式/默认 `readable-view` 和 `protocol-json` 的 read 输出边界；不枚举所有 operation。 |
| `MD-MACHINE-001` | [machine-commands.ts](../../test/smoke/markdown/cases/machine-commands.ts) | `manifest`、`probe` 和 valid `invoke` 的机器协议链路；证明 metadata、format support 和 stdin protocol request 可通过 schema。 |
| `MD-CORPUS-001` | [corpus.ts](../../test/smoke/markdown/cases/corpus.ts) | Unicode 文档的 outline/read 和分页重组；其它编码/换行 fixture 属于较低层测试。 |
| `MD-ARGS-001` | [cli-args.ts](../../test/smoke/markdown/cases/cli-args.ts) | 当前 operation 实际使用参数缺失时返回输入错误；其它同类非法值属于 parser 单测。 |
| `MD-WARN-001` | [cli-args.ts](../../test/smoke/markdown/cases/cli-args.ts) | document help、readable-json warning、unused native flag warning、protocol-json stderr warning；不枚举 token 组合。 |
| `MD-ERROR-001` | [operation-errors.ts](../../test/smoke/markdown/cases/operation-errors.ts) | 同一 invalid ref 错误在 readable-json 与 protocol-json 中的映射；其它错误码由 lower-level tests 覆盖。 |
| `MD-INVOKE-001` | [invoke-errors.ts](../../test/smoke/markdown/cases/invoke-errors.ts) | malformed `invoke` stdin 返回 protocol failure；证明 stdin request 解析失败时仍返回稳定 protocol error envelope。 |

## 新增或修改用例

新增 case 前先确认它代表新的外部链路类型。若只是已有类型中的另一组参数、另一个非法值、另一个 output/operation 组合或另一个同阶段错误，应优先补充较低层测试或扩展现有 case 的断言。
