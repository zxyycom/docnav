**一句话核心：实现独立的 Markdown v0 adapter，让 `outline -> ref -> read` 纵向链路先在真实格式上跑通。**

## Why

v0 首期明确聚焦 Markdown，且 Markdown adapter 必须实现 `outline`、`read`、`find` 和 `info` 全部能力。协议地基完成后，需要一个真实 adapter 验证扁平 outline、唯一 ref、字符预算、page 和直接 CLI 输出。

## What Changes

- 新增 `docnav-markdown` 独立可执行 adapter，提供 `manifest`、`probe`、`invoke` 和直接 CLI 命令。
- 实现 Markdown `outline`、`read`、`find`、`info` 全部能力。
- 使用成熟 Markdown parser，遵守 heading、章节范围、frontmatter、代码围栏、重复 heading、非 UTF-8 和分页行为约束。
- 生成 adapter 拥有的可读唯一 ref，并在 read 中唯一定位对应区域。
- 非目标：本 change 不实现跨格式 adapter 路由、不实现 `docnav adapter install/update/remove/list`、不实现 MCP bridge、不实现 JSON/YAML/TOML/INI adapter。

## Capabilities

### New Capabilities

- `markdown-adapter-v0-implementation`: 实现 Markdown adapter 的 manifest/probe/invoke、四项文档能力、ref 生成解析、分页和直接 CLI 输出。

### Modified Capabilities

- 无。

## Impact

- 影响 adapter 制品：`docnav-markdown`。
- 影响共享协议使用面：通过 `docnav-adapter-sdk` 输出完整 protocol envelope，并提供 readable/text 直接 CLI 输出。
- 影响测试：新增 Markdown parser、outline/read/find/info、ref 唯一性和 page 的单元与 invoke 测试。
