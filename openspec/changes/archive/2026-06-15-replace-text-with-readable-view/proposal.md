本 change 的目标是用仓库内 renderer config 驱动的 `readable-view` 替代 document operation 的 `text` 输出模式。

## Why

当前 document `text` 输出由各 operation 和 adapter 分别拼接字段，容易遗漏 warning、page、content type 或后续新增字段，也无法从输出中审计正文原本位于哪个 readable 字段。`readable-json` 保留完整结构，但多行 Markdown 被转义为 JSON 字符串，不适合作为默认直接阅读形态。

`readable-view` 的目标是让默认阅读输出同时满足两个要求：

1. header 保留 readable payload 的字段结构、字段位置和 warning/error 信息。
2. Markdown 正文等指定字段以原文 block 输出，避免 JSON 字符串转义影响阅读。

## What Changes

- 新增 `readable-view` 阅读输出：先输出合法 pretty JSON header，再输出由 header 中显式 block 引用定位的原文 block。
- 新增仓库内、随代码发布的 renderer config，用于声明每个 readable view kind 的 block 字段；block 字段集合由仓库代码拥有，不进入 CLI、项目配置、用户配置或 adapter manifest。
- 所有 document operation、readable error 和 warning 使用同一个通用 renderer；没有 block 字段的结果仍输出同一 readable-view header，operation 展示差异通过 typed readable payload 和 renderer config 表达。
- `readable-view` 和 `readable-json` 从同一个 typed readable result 派生；`readable-json` schema 和完整字段语义保持不变。
- `docnav` 和 adapter direct CLI 的 document operation 默认输出改为 `readable-view`。
- MCP structuredContent 继续从 `readable-json` 派生。当前 change 只提供 readable-view 契约、仓库内 renderer config 和 conformance vectors；MCP TextContent 的 JavaScript renderer 和 bridge 接入由 `implement-docnav-mcp-bridge` change 承接。
- **BREAKING** document output surface 收敛到 readable-view/readable-json/protocol-json 与共享 renderer path；核心 `OutputMode::Text`、adapter `DirectOutputMode::Text`、独立 document text formatter、document text 模板配置和 document text golden/smoke 契约收敛为当前 readable 验收材料。非文档命令的 help、version 和其它纯文本诊断保留独立 `PlainText` 输出通道。
- **BREAKING** document operation 输出模式固定为 `readable-view`、`readable-json` 和 `protocol-json`；help、README、主规范、schema/example 索引、测试矩阵、skills 和在途 change 同步声明这三种当前模式。
- 已有 `defaults.output` 配置 key 的合法值收敛到三种当前 document output mode；其它值按通用配置错误报告。

## Capabilities

### New Capabilities

- `readable-view-output`: 定义 readable-view 仓库内 renderer config、静态 block 字段契约、block 引用、原文 block framing、默认输出、renderer 失败语义和与 readable-json 的同源关系。

### Modified Capabilities

- `core-cli`: 将核心 CLI document 默认输出和 warning/error 阅读映射切换为 readable-view，document output mode 固定为 readable-view/readable-json/protocol-json，并保留非文档 `PlainText` 通道。
- `adapter-protocol`: 让 adapter direct CLI 复用通用 readable-view renderer，SDK document output surface 使用三种当前模式和 shared readable payload。
- `markdown-navigation`: 将 Markdown direct CLI 与黑盒 smoke 的阅读输出验收改为 readable-view，并通过 SDK shared renderer path 承载展示。

## Impact

- 受影响制品：`docnav`、`docnav-adapter-sdk`、`docnav-markdown`、readable-view renderer config/golden fixtures、`implement-docnav-mcp-bridge` 的待同步 artifacts。
- 受影响 public surface：document operation `--output` 枚举、默认输出、`defaults.output` 配置、help、stdout readable contract 和错误/warning 展示。
- 受影响验证材料：`docs/cli.md`、`docs/architecture.md`、`docs/adapter-contract.md`、`docs/protocol.md`、`docs/testing.md`、`docs/CODING_STYLE.md`、README、readable view golden fixtures、CLI smoke 和 Rust output tests。
- `readable-json`、`protocol-json`、adapter `invoke` envelope、readable JSON schema、ref、pagination、adapter routing 语义、help/version 的纯文本输出语义不变。
- `explore-operation-composition` 只记录未来 operation composition 方向，不定稿具体默认输出；后续 implementation change 按本 change 的 typed readable shape、renderer config 和 readable-view contract 设计。`implement-docnav-mcp-bridge` 在其自身 change 中消费本 change 提供的 renderer config/conformance vectors。
- 非目标：结构化消费入口继续使用 readable-json 或 protocol-json；block 字段集合由仓库 renderer config 固定；MCP JavaScript renderer 在 `implement-docnav-mcp-bridge` change 中实现。
