## Why

Docnav 的核心 CLI、adapter 直接 CLI 和后续 MCP 映射都需要在命令行参数扩展时保持前向兼容。当前 adapter 直接 CLI 仍把未知 flag、多余 positional 和当前 operation 不使用的已知 flag 作为输入错误处理，和核心 `docnav` 的兼容性参数策略不一致。

## What Changes

- 统一所有 Docnav 直接 CLI 的兼容参数处理：未知 flag、多余 positional 和当前 operation 不使用的已知 flag 生成 warning 后继续执行。
- 明确 warning token 归属：每条 warning 记录原始被忽略 argv token、kind 和 reason；`--unknown=value` 归为一个未知 flag token，`--unknown value` 中的 `value` 继续普通解析。
- 按输出模式承载 warning：可读文本在正常结果后拼接 warning；JSON 和其它 structured 输出增加 `warnings` 键；有独立诊断通道的 CLI 可将同一 warning 同步写入 stderr。
- 保留已知必需参数缺失、已知 flag 缺少值或值非法的稳定错误行为。
- 将兼容性参数解析下沉到共享 SDK，使格式 adapter 可以复用同一规则。
- 更新 Markdown adapter 直接 CLI 和 smoke 测试，覆盖 unknown flag、多余 positional、当前 operation 不使用的已知 flag、unknown flag 不吞后续 token 的 warning 行为。
- 保持 invoke 协议 schema 不变；adapter `invoke` stdin JSON 仍严格校验 request JSON 和 operation arguments。

## Capabilities

### New Capabilities

- 无。

### Modified Capabilities

- `protocol-and-adapter-sdk-implementation`: 共享 SDK 直接 CLI 参数解析改为兼容 CLI 扩展参数，并输出可验证 warning。
- `markdown-adapter-v0-implementation`: Markdown adapter 直接 CLI 采用 SDK 兼容性参数规则，并更新黑盒 smoke 断言。

## Impact

- 影响 `docnav-adapter-sdk` 直接 CLI 参数解析。
- 影响 `docnav-markdown` 直接 CLI 行为和 smoke 测试。
- 影响 CLI 主规范、adapter 契约和测试策略中关于 unknown flag、多余 positional、warning 承载、输出字段和退出码的描述。
- 不影响原始 invoke 协议字段、schema、ref 语义或 Markdown parser。
