## Context

当前 `docnav` 核心 CLI 已计划对 CLI 扩展参数采用 warning 后忽略的前向兼容策略，但 adapter 直接 CLI 仍通过 SDK 将 unknown flag、当前 operation 不使用的已知 flag 和多余 positional 作为输入错误处理。这个差异会让同一类命令行扩展在不同入口表现不一致，并增加后续 adapter 维护成本。

## Goals / Non-Goals

**Goals:**

- 让所有 Docnav 直接 CLI 对未知 flag、多余 positional 和当前 operation 不使用的已知 flag 使用同一兼容策略。
- 通过 `docnav-adapter-sdk` 提供共享参数解析能力，使 adapter 直接 CLI 复用该策略。
- 明确 token 归属：warning 指向实际被忽略的原始 argv token；`--unknown=value` 是一个 token；`--unknown value` 只把 `--unknown` 归 unknown flag，`value` 继续普通解析。
- 保持输出模式有效：文本输出在正常结果后拼接 warning；JSON 和其它 structured 输出通过 `warnings` 键承载 warning，并保持 stdout payload 合法。
- warning 必须列出具体被忽略 token、kind 和 reason。
- 保留已知必需参数缺失、已知 flag 缺少值或值非法的稳定错误行为。

**Non-Goals:**

- 保持 adapter `invoke` stdin JSON request schema 严格校验。
- 不为格式 adapter 增加新的格式专属参数语义。

## Decisions

1. SDK 拥有通用兼容参数解析。
   - `docnav-adapter-sdk` 的直接 CLI 参数解析识别通用 flag、operation 必需参数和 adapter 声明的格式专属 flag。
   - 未识别 flag、多余 positional 和当前 operation 不使用的已知 flag 都生成 warning 后忽略。
   - warning item 必须包含 `ignored_tokens`、`kind` 和 `reason`；`ignored_tokens` 使用原始 argv token 文本，单 token 忽略时数组长度为 1。
   - warning 不改变原本可成功执行命令的退出码。

2. 未知 flag 不吞后续 token。
   - 未知 flag 只忽略该 flag token 本身。
   - `--unknown=value` 作为一个未知 token 被忽略。
   - `--unknown value` 中的 `value` 继续按普通 token 处理：它可以填充仍可用的 positional 槽位，也可以在没有槽位接收时作为多余 positional 单独 warning。
   - 因此 `--future --output protocol-json` 必须仍然解析 `--output protocol-json`。

3. 已知 flag 的取值按“紧跟 token”解析。
   - 对需要值的已知 flag，下一个 token 就是值，即使它以 `--` 开头。
   - 只有没有下一个 token 时，才判定为该已知 flag 缺少值并返回 `INVALID_REQUEST` / 输入错误。
   - 对无值 flag，后续 token 继续按普通参数处理。
   - 对当前 operation 不使用的已知有值 flag，SDK 按该 flag 的形状消费紧跟 value token，并在 warning 的 `ignored_tokens` 中同时记录 flag token 和被消费的 value token。

4. warning 按输出模式承载。
   - text 输出在正常结果后拼接 warning 文本；warning 文本打印被忽略 token 和 reason。
   - readable-json、protocol-json 和其它 structured stdout payload 增加顶层 `warnings` 数组字段。
   - 有独立诊断通道的 CLI 可以把同一 warning 同步写 stderr；stdout 始终保持所选输出模式的合法 payload。

5. invoke 入口保持严格。
   - `adapter invoke` 不接受命令行 positional 参数；多余 positional 仍作为 invoke 入口错误。
   - invoke stdin JSON 继续由 protocol request schema 严格校验，未知字段和参数类型错误返回结构化协议失败。

6. Markdown adapter 只更新 CLI wrapper 行为。
   - Markdown parser、ref、分页、manifest、probe 和 operation handler 不变化。
   - smoke 矩阵中 unknown flag、当前 operation 不使用的已知 flag 和 extra positional 改为 warning 兼容用例；缺 path、缺 ref/query、非法数字和非法 max heading level 仍保持失败。

## Risks / Trade-offs

- [调用方拼错参数但命令仍成功] → 每个被忽略参数产生包含 token 的 warning，并在 smoke 中断言 text/JSON 都按各自输出模式承载。
- [SDK 与 core CLI 规则漂移] → 把取值规则写入 SDK spec，并在 core CLI 和 Markdown adapter smoke 中都覆盖。
- [invoke 严格性被误放宽] → 明确兼容策略只适用于直接 CLI 参数，不适用于 invoke stdin JSON。
