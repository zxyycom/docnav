本 change 的目标是用仓库内版本化 renderer config 驱动的 `readable-view` 替代 document operation 的 `text` 输出模式；本文是未审核临时 proposal，只存在于 `openspec/changes/replace-text-with-readable-view/`，不改变现有主规范或其它文档。

## Why

当前 document `text` 输出由各 operation 和 adapter 分别拼接字段，容易遗漏 warning、page、content type 或后续新增字段，也无法从输出中审计正文原本位于哪个 readable 字段。`readable-json` 保留完整结构，但多行 Markdown 被转义为 JSON 字符串，不适合作为默认直接阅读形态。

`readable-view` 的目标是让默认阅读输出同时满足两个要求：

1. header 保留 readable payload 的字段结构、字段位置和 warning/error 信息。
2. Markdown 正文等指定字段以原文 block 输出，避免 JSON 字符串转义影响阅读。

## What Changes

- 新增 `readable-view` 阅读输出：先输出带格式版本标识的合法 JSON header，再输出由 header 中显式 block 引用定位的原文 block。
- 新增仓库内、版本化、随代码发布的 renderer config，用于声明每个 readable view kind 的格式版本和 block 字段；该配置不是用户配置，调用方不能通过 CLI、项目配置、用户配置或 adapter manifest 改变。
- 所有 document operation、readable error 和 warning 使用同一个通用 renderer；没有 block 字段的结果仍输出同一 readable-view header，不建立 operation 私有文本模板。
- `readable-view` 和 `readable-json` 从同一个 typed readable result 派生；`readable-json` schema 和完整字段语义保持不变。
- `docnav` 和 adapter direct CLI 的 document operation 默认输出改为 `readable-view`。
- MCP structuredContent 继续从 `readable-json` 派生。当前 change 只提供 readable-view 契约、仓库内 renderer config 和 conformance vectors；MCP TextContent 的 JavaScript renderer 和 bridge 接入由 `implement-docnav-mcp-bridge` change 承接。
- **BREAKING** 删除 document operation 的 `--output text`、`OutputMode::Text`、adapter `DirectOutputMode::Text`、独立 document text formatter、document text 模板配置和所有 document text golden/smoke 契约。非文档命令的 help、version 和其它纯文本诊断保留独立 `PlainText` 输出通道。
- **BREAKING** document operation 输出模式只保留 `readable-view`、`readable-json` 和 `protocol-json`；help、README、主规范、schema/example 索引、测试矩阵、skills 和在途 change 中不得继续把 document `text` 描述为受支持模式。
- 为 legacy `defaults.output: "text"` 定义可操作修复路径：普通 document execution 必须拒绝该配置值；`docnav config set defaults.output readable-view` 和 `docnav config unset defaults.output` 必须能在目标配置包含 legacy text 时完成修复。

## Capabilities

### New Capabilities

- `readable-view-output`: 定义 readable-view 格式版本、仓库内 renderer config、静态 block 字段契约、block 引用、原文 block framing、默认输出、renderer 失败语义和与 readable-json 的同源关系。

### Modified Capabilities

- `core-cli`: 将核心 CLI document 默认输出和 warning/error 阅读映射切换为 readable-view，移除 document text 模式、默认值和静默兼容，并保留非文档 `PlainText` 通道。
- `adapter-protocol`: 让 adapter direct CLI 复用通用 readable-view renderer，移除 SDK document text output mode、text formatter 和 text warning 拼接路径。
- `markdown-navigation`: 将 Markdown direct CLI 与黑盒 smoke 的 text 验收替换为 readable-view 验收，并删除 Markdown 私有文本 formatter。

## Impact

- 受影响制品：`docnav`、`docnav-adapter-sdk`、`docnav-markdown`、readable-view renderer config/golden fixtures、`implement-docnav-mcp-bridge` 的待同步 artifacts。
- 受影响 public surface：document operation `--output` 枚举、默认输出、`defaults.output` 配置、help、stdout readable contract 和错误/warning 展示。
- 受影响验证材料：`docs/cli.md`、`docs/architecture.md`、`docs/adapter-contract.md`、`docs/protocol.md`、`docs/testing.md`、`docs/CODING_STYLE.md`、README、readable view golden fixtures、CLI smoke 和 Rust output tests。
- `readable-json`、`protocol-json`、adapter `invoke` envelope、readable JSON schema、ref、pagination、adapter routing 语义、help/version 的纯文本输出语义不变。
- `add-fast-outline` 必须使用 readable-view 替代其中的默认文本约定；`implement-docnav-mcp-bridge` 必须在其自身 change 中消费本 change 提供的 renderer config/conformance vectors，并删除独立 text formatter 语义。
- 非目标：不引入 YAML/Hjson/TOML 作为新机器协议，不允许调用方动态选择 block 字段，不把 readable-view 提升为长期机器兼容接口，不在本 change 内实现 MCP JavaScript bridge renderer。
