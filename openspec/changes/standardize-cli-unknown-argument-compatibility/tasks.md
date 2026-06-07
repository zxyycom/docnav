## 1. SDK 参数解析

- [x] 1.1 在 `docnav-adapter-sdk` 直接 CLI 参数解析中区分已知必需参数、已知有值 flag、已知无值 flag、未知 flag 和多余 positional。
- [x] 1.2 实现未知 flag、多余 positional 和当前 operation 不使用的已知 flag 的 warning 后忽略行为；warning item 必须包含 `ignored_tokens`、`kind` 和 `reason`。
- [x] 1.3 实现未知 flag 不吞后续 token：`--unknown=value` 作为一个 token 忽略；`--unknown value` 中的 `value` 继续按普通 token 处理，可填充 positional 槽位或作为多余 positional 单独 warning。
- [x] 1.4 实现已知有值 flag 的紧跟 token 取值规则：下一个 token 即为值，即使以 `--` 开头；只有无下一个 token 时返回缺值错误。
- [x] 1.5 实现 warning 输出承载：text 在正常结果后拼接 warning，readable-json 和 MCP 等阅读层 structured 输出增加顶层 `warnings` 数组。
- [x] 1.6 确保 protocol-json、manifest 和 probe stdout 不增加 `warnings` 字段；存在 CLI warning 时必须写入 stderr，stdout 仍通过对应 schema。
- [x] 1.7 保持 adapter `invoke` stdin JSON 严格 schema 校验，不复用直接 CLI 的兼容忽略规则。

## 2. Markdown Adapter 接入

- [x] 2.1 让 `docnav-markdown` 直接 CLI 通过 SDK 兼容参数解析处理文档操作、manifest 和 probe 命令。
- [x] 2.2 更新 Markdown CLI 参数测试用例：unknown flag、多余 positional 和当前 operation 不使用的已知 flag 改为 warning 后继续，并断言 warning 包含具体 `ignored_tokens`、kind 和 reason。
- [x] 2.3 保留缺 path、缺 `--ref`、缺 `--query`、非法 page、非法 limit_chars 和非法 max_heading_level 的非零退出断言。
- [x] 2.4 覆盖 unknown flag 不吞后续 token 场景：`--future --output protocol-json` 仍解析 output，`--future value` 中的 `value` 按普通 token 归属。
- [x] 2.5 覆盖 `--ref --future-value` 这类已知 flag 紧跟 token 取值场景。

## 3. 文档同步

- [x] 3.1 同步 `docs/cli.md`、`docs/adapter-contract.md` 和 `docs/testing.md`，明确兼容策略适用于所有直接 CLI 参数，不适用于 invoke stdin JSON，并定义 text/readable-json/protocol-json warning 承载边界。
- [x] 3.2 更新 `docs/references/markdown-navigator.md` 中关于未知参数旧行为的说明，避免继续声明 adapter CLI 必须失败。
- [x] 3.3 更新 readable JSON schema、MCP output schema 和相关示例，使阅读层 structured 输出允许 `warnings`；不得向 protocol response、manifest 或 probe schema 增加 CLI warning 字段。
- [x] 3.4 修正当前 core CLI change 中的兼容参数规则，使其与本 change 的 SDK 规则和 protocol-json warning 边界一致。

## 4. 验证

- [x] 4.1 运行 Markdown CLI smoke，验证 warning 按输出模式承载、warning 说明具体忽略 token 和原因、protocol-json/manifest/probe stdout 仍通过 schema、成功路径退出码不变、负向输入仍按规则失败。
- [x] 4.2 运行 Rust SDK 和 Markdown adapter 相关测试，覆盖直接 CLI 参数解析和 invoke 严格校验。
- [x] 4.3 运行 schema 和示例校验，确认 readable 层允许 warnings 且 protocol/manifest/probe 层不接受 CLI warning 字段。
- [x] 4.4 运行 `openspec validate standardize-cli-unknown-argument-compatibility --strict`。
- [x] 4.5 若本 change 与 core CLI change 同时交付，最终运行 `pnpm run verify:docnav-workspace`。
