本 change 目标是将分页默认值统一收敛到 `defaults.pagination`，并让 core `docnav` 和 adapter SDK direct CLI 使用同一 pagination 配置/argv 行为；本文档只是 `openspec/changes/configure-pagination-defaults/` 下的未审核临时 proposal，不影响现有其它文档或主规范。

## Why

当前分页配置缺少统一的 enabled 状态，配置文件和显式 argv 无法用同一套参数来源表达“本次是否启用分页”。

需要把“分页是否启用”和“分页启用时的字符预算”合并为一个 pagination 默认值对象，让 core CLI 和 adapter SDK direct CLI 在进入分页前用同一规则初始化最终 `limit_chars`。

## What Changes

- **BREAKING**：核心配置和 adapter direct CLI 配置的分页默认值改用 `defaults.pagination.enabled` 与 `defaults.pagination.limit_chars`。
- `defaults.pagination.enabled` 默认为 `true`；为 `false` 时，对外语义是不启用分页，内部在进入 invoke 或 operation handler 前把最终 `limit_chars` 初始化为协议 `PositiveInteger` 可表示的最大值。
- `defaults.pagination.limit_chars` 是分页启用时的正整数字符预算，默认保持当前 `6000`。
- `page` 仍不是配置默认值；省略时固定为 `1`，返回的非 null `page` 仍表示下一页。
- Core `docnav` document commands 和 adapter SDK direct CLI document commands 新增显式 argv `--pagination enabled|disabled`，并映射为 `defaults.pagination.enabled` 的显式来源。
- 显式 argv `--limit-chars` 映射为 `defaults.pagination.limit_chars` 的显式来源；它与 `--pagination` 共同进入 pagination 参数归一流程。
- Core `docnav` 和 adapter SDK direct CLI 必须对同名 pagination canonical key 使用一致的 config path、flag semantics、source priority、validation semantics 和 disabled finalization 规则。
- Adapter `invoke` stdin JSON 仍保持严格协议输入，不读取 direct CLI 配置，也不新增 protocol 字段。

## Capabilities

### New Capabilities

- 无新增 capability ID。

### Modified Capabilities

- `core-cli`：修改 core 配置域和默认值解析，支持 `defaults.pagination` 并生成最终 `limit_chars`。
- `adapter-protocol`：修改 adapter SDK direct CLI 的 pagination 配置投影、argv 覆盖和分页 helper 使用边界，让 SDK direct CLI 支持同一 pagination 配置对象。
- `markdown-navigation`：更新 Markdown adapter direct CLI 的配置 schema、示例和测试期望，使其通过 SDK direct CLI 配置处理消费 `defaults.pagination`。

## Impact

- 影响 `docnav` core pagination 配置模型、document command parser、`config get/set/unset/list`、document context 默认值输出、CLI help/default 文案和相关测试。
- 影响 `docnav-adapter-sdk` direct CLI pagination argv parser、配置读取、参数来源对象、配置源合并、help/default 文案和 SDK tests。
- 影响 `docnav-markdown` adapter direct CLI 配置 schema/example、smoke/matrix 测试和文档说明。
- 影响 `docs/cli.md`、`docs/adapter-contract.md`、`docs/protocol.md`、`docs/adapters/markdown.md`、`docs/schemas/docnav-markdown-config.schema.json`、`docs/examples/json/docnav-markdown-config.json` 以及测试策略中涉及配置与分页的验证描述。
- 不影响 protocol request/result 字段形状；`limit_chars` 和 `page` 仍是 invoke 请求中的显式正整数参数。
