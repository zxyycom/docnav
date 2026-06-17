# Docnav Markdown CLI Smoke Cases

本目录只放 `docnav-markdown` 直接 CLI 的真实进程边界 smoke。用例按外部链路类型管理，不按 operation、output mode、非法参数或 fixture 做笛卡尔积扩展；同一类型只保留一条能证明边界行为的代表用例。细粒度 Markdown 解析、ref grammar、分页边界、配置默认值和 schema 失败应下沉到 Rust 单元/集成测试或 schema validators。

连续链路可以在一个 case 内串行执行多个 CLI 命令；只有独立链路才拆成独立 task 调度。

| Case ID | 文件 | 验证目标 | 保留原因 |
| --- | --- | --- | --- |
| `MD-LINK-001` | `outputs.mjs` | `outline -> ref -> read`、`find -> ref -> read` 和 `info` 的 readable-json 串行链路。 | 证明 Markdown adapter 的直接 CLI 主链路、ref 生成/读取和 readable-json 输出分层。 |
| `MD-OUTPUT-001` | `outputs.mjs` | `readable-json`、显式/默认 `readable-view` 和 `protocol-json` 的 read 输出边界。 | 证明三种 document output mode 的 envelope、block framing 和默认输出行为，不枚举所有 operation。 |
| `MD-MACHINE-001` | `machine-commands.mjs` | `manifest`、`probe` 和 valid `invoke` 的机器协议链路。 | 证明 adapter metadata、format support 和 stdin protocol request 均可通过 schema 并保持 read 等价。 |
| `MD-CORPUS-001` | `corpus.mjs` | Unicode 文档的 outline/read 和分页重组。 | 证明真实进程边界下 UTF-8 内容和 continuation page 可用；其它编码/换行 fixture 属于较低层测试。 |
| `MD-ARGS-001` | `cli-args.mjs` | 当前 operation 实际使用参数缺失时返回输入错误。 | 证明直接 CLI 参数严格失败路径；其它同类非法值属于 parser 单测。 |
| `MD-WARN-001` | `cli-args.mjs` | document help、readable-json warning、unused native flag warning、protocol-json stderr warning。 | 证明兼容 warning 的输出位置和 stable warning envelope，不枚举 token 组合。 |
| `MD-ERROR-001` | `operation-errors.mjs` | 同一 invalid ref 错误在 readable-json 与 protocol-json 中的映射。 | 证明 adapter-owned ref 错误语义和输出层包装边界；其它错误码由 lower-level tests 覆盖。 |
| `MD-INVOKE-001` | `invoke-errors.mjs` | malformed `invoke` stdin 返回 protocol failure。 | 证明 stdin request 解析失败时仍返回稳定 protocol error envelope。 |

新增用例时先判断它是否代表新的外部链路类型。若只是已有类型中的另一组参数、另一个非法值、另一个 output/operation 组合或另一个同阶段错误，应优先补充较低层测试或扩展现有 case 的断言，而不是新增 task。
