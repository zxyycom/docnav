## Why

Docnav 会持续扩展 CLI 参数。核心 `docnav` 已采用“warning 后继续”的前向兼容策略，但 adapter 直接 CLI 仍把未知 flag、多余 positional 和当前 operation 不使用的已知 flag 当作输入错误。

这个差异会让同一类命令在不同入口表现不一致，并让后续 MCP、adapter 管理和新增格式 adapter 重复处理参数兼容逻辑。本 change 将兼容规则收敛为一个可复用、可测试的直接 CLI 契约。

## What Changes

- 定义所有 Docnav 直接 CLI 的兼容参数规则：未知 flag、多余 positional 和当前 operation 不使用的已知 flag 生成 warning 后继续执行。
- 将直接 CLI 参数解析下沉到 `docnav-adapter-sdk`，由 SDK 统一识别已知必需参数、已知有值 flag、已知无值 flag、未知 flag、当前 operation 不使用的已知 flag 和多余 positional。
- 规定 token 归属：warning 记录原始被忽略 argv token、kind 和 reason；`--unknown=value` 作为一个未知 flag token；`--unknown value` 只忽略 `--unknown`，`value` 继续参与普通解析。
- 规定已知有值 flag 的取值：紧跟该 flag 的下一个 token 就是值，即使该 token 以 `--` 开头；只有没有下一个 token 时才返回缺值错误。
- 规定 warning 承载边界：text 在正常结果后拼接 warning；`readable-json` 和 MCP 等阅读层 structured 输出增加 `warnings`；`protocol-json`、`manifest` 和 `probe` stdout 保持 schema-valid JSON，warning 写入 stderr。
- 保留硬错误：已知必需参数缺失、已知 flag 缺少值或值非法仍返回输入错误。
- 更新 Markdown adapter 直接 CLI 和黑盒 smoke，覆盖兼容 warning、unknown flag 不吞后续 token、已知 flag 紧跟 token 取值和 invoke 严格校验。
- 同步核心 CLI change 的参数兼容规则，使当前主要改动成为该规则的所有者。

## Capabilities

### New Capabilities

- 无。

### Modified Capabilities

- `protocol-and-adapter-sdk-implementation`: 共享 SDK 直接 CLI 参数解析改为兼容 CLI 扩展参数，并输出可验证 warning。
- `markdown-adapter-v0-implementation`: Markdown adapter 直接 CLI 采用 SDK 兼容性参数规则，并更新黑盒 smoke 断言。

## Impact

- 直接影响 `docnav-adapter-sdk` 参数解析、`docnav-markdown` 直接 CLI 行为和 Markdown CLI smoke。
- 需要同步 `docs/cli.md`、`docs/adapter-contract.md`、`docs/testing.md`、readable JSON schema、MCP readable schema 和相关示例中的 warning 字段说明。
- 需要同步当前核心 CLI change，使 `docnav` 与 adapter 直接 CLI 使用同一 warning 规则和 protocol-json 边界。
- 不改变 adapter `invoke` 请求/响应 schema、protocol response schema、manifest schema、probe schema、ref 语义或 Markdown parser。
