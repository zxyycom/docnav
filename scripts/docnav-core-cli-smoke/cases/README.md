# Docnav Core CLI Smoke Cases

本目录只放 core CLI 的真实进程边界 smoke。用例按外部链路类型管理，不按参数笛卡尔积扩展；同一类型只保留一条能证明边界行为的代表用例。细粒度 parser、config、renderer、schema 和 adapter 语义应下沉到 Rust 单元/集成测试或 adapter smoke。

连续链路可以在一个 case 内串行执行多个 CLI 命令；只有独立链路才拆成独立 task 调度。

| Case ID | 文件 | 验证目标 | 保留原因 |
| --- | --- | --- | --- |
| `CORE-LINK-001` | `real-markdown.mjs` | 真实 `docnav` + 真实 `docnav-markdown` 的 `outline -> ref -> read`、`find -> ref -> read` 与 `info` 串行链路。 | 证明 core 不解析 ref，只原样传递 adapter 生成的 ref，并保持 readable-json 输出分层。 |
| `CORE-REF-001` | `real-markdown.mjs` | 真实 markdown adapter 返回的 ref 错误映射为 `protocol-json` failure。 | 证明 adapter-owned ref 错误能穿过 core 映射为稳定协议错误。 |
| `CORE-OUTPUT-001` | `outputs.mjs` | `readable-json`、显式/默认 `readable-view`、`protocol-json` 和 warning 承载边界。 | 证明三种 document output mode 的通道、envelope 和 block framing 边界，不枚举所有 operation。 |
| `CORE-ARGS-001` | `cli-args.mjs` | 当前 operation 实际使用参数缺失时返回 protocol failure。 | 证明 CLI 参数严格失败路径和 `operation` 绑定；其它同类非法值属于 parser/config 单测。 |
| `CORE-CONFIG-001` | `config-management.mjs` | project/user config 优先级、非法 output 值、`config list --path` 的 adapter/path context。 | 证明 core 配置域会影响最终 defaults 和 path context；不枚举全部 config get/set/unset 组合。 |
| `CORE-SELECT-001` | `adapter-selection.mjs` | 显式预选 adapter manifest 失败后继续 registry fallback，并在 readable-json 中输出 candidate warning。 | 证明 adapter 选择链路的 recoverable candidate failure 和 fallback 行为。 |
| `CORE-FAIL-001` | `failures.mjs` | candidate 阶段 adapter 进程失败被记录为 `FORMAT_UNKNOWN` candidate evidence。 | 证明 registry/candidate 失败证据 shape，避免逐项枚举 manifest/probe/contract/process 组合。 |
| `CORE-INVOKE-001` | `failures.mjs` | invoke 阶段 adapter 进程失败映射为 `ADAPTER_INVOKE_FAILED`。 | 证明 selected adapter invoke failure 与 candidate failure 属于不同错误阶段。 |
| `CORE-TOOLS-001` | `config-management.mjs` | `init`、`version`、`doctor` 和 document help 的可用性。 | 证明非 document operation 的主要入口可运行，并保留 output mode help smoke。 |

新增用例时先判断它是否代表新的外部链路类型。若只是已有类型中的另一组参数、另一个非法值、另一种同阶段失败，应优先补充较低层测试或扩展现有 case 的断言，而不是新增 task。
