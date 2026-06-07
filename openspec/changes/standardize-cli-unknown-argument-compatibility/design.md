## Context

当前核心 CLI change 已计划对 CLI 扩展参数采用 warning 后继续的前向兼容策略。adapter 直接 CLI 仍通过 SDK 将 unknown flag、当前 operation 不使用的已知 flag 和多余 positional 作为输入错误处理。

本 change 将“直接 CLI 参数兼容”提升为共享规则：核心 CLI 跟随该规则，格式 adapter 通过 SDK 复用该规则；协议形 stdout 和 invoke stdin 继续保持严格 schema 边界。

## Goals / Non-Goals

**Goals:**

- 为所有 Docnav 直接 CLI 建立同一套兼容参数规则。
- 让 `docnav-adapter-sdk` 提供共享解析能力，adapter 直接 CLI 通过 SDK 复用规则。
- 让 warning 可审计：每条 warning 明确 `ignored_tokens`、`kind` 和 `reason`。
- 让 token 归属可预测：未知 flag 不吞后续 token，已知有值 flag 固定消费紧跟 token。
- 让输出层边界可验证：阅读层可以承载 structured warnings，协议形 stdout 继续通过原 schema。
- 保留稳定失败：已知必需参数缺失、已知 flag 缺少值或值非法仍失败。

**Non-Goals:**

- adapter `invoke` stdin JSON request schema 继续严格校验；本 change 只处理直接 CLI argv 兼容。
- 不为格式 adapter 增加新的格式专属参数语义。
- 不修改 adapter `invoke`、CLI `protocol-json`、`manifest` 或 `probe` 的协议 schema 来承载 CLI warning。

## Decisions

1. SDK 拥有通用兼容参数解析。
   - `docnav-adapter-sdk` 识别通用 flag、operation 必需参数和 adapter 声明的格式专属 flag。
   - 未识别 flag、多余 positional 和当前 operation 不使用的已知 flag 生成 warning 后忽略。
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
   - readable-json 和 MCP 等阅读层 structured payload 增加顶层 `warnings` 数组字段。
   - protocol-json、manifest 和 probe 属于协议或专属机器 schema 输出，stdout 不增加 `warnings` 字段；存在 CLI warning 时必须写入 stderr，stdout 仍只包含一个可通过对应 schema 的 JSON 值。

5. invoke 入口保持严格。
   - `adapter invoke` 不接受额外命令行 positional 参数；多余 positional 仍作为 invoke 入口错误。
   - invoke stdin JSON 继续由 protocol request schema 严格校验，未知字段和参数类型错误返回结构化协议失败。

6. Markdown adapter 只更新 CLI wrapper 行为。
   - Markdown parser、ref、分页、manifest、probe 和 operation handler 不变化。
   - smoke 矩阵中 unknown flag、当前 operation 不使用的已知 flag 和 extra positional 改为 warning 兼容用例；缺 path、缺 ref/query、非法数字和非法 max heading level 仍保持失败。

## Risks / Trade-offs

- [调用方拼错参数但命令仍成功] → 每个被忽略参数产生包含 token、kind 和 reason 的 warning，并在 smoke 中断言 text/readable-json/protocol-json 按各自边界承载。
- [SDK 与 core CLI 规则漂移] → 把取值规则写入 SDK spec，并在 core CLI 和 Markdown adapter smoke 中都覆盖。
- [invoke 严格性被误放宽] → 明确兼容策略只适用于直接 CLI 参数，不适用于 invoke stdin JSON。
- [协议形 stdout schema 保持稳定] → protocol-json、manifest 和 probe stdout 保持 schema-valid，CLI warning 只进入 stderr；readable-json/MCP 才把 warning 作为 structured 字段承载。
