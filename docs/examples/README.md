# 端到端示例

本目录使用 [nested-duplicate.md](markdown/nested-duplicate.md) 展示 Markdown `outline -> ref -> read`、page、原始协议与阅读输出的边界。

## 关键原则

- invoke 返回包含 operation 的完整 protocol envelope。
- `docnav` 是识别格式、选择 adapter 和映射输出的核心 CLI。
- MCP bridge 目标 structuredContent 返回精简 readable 结果，不包含 envelope；直接 CLI warning 触发规则见 [标准参数](../standard-parameters.md#错误出口)，warning item 使用 `id`、非空 `reason`、稳定 `effect` 和 `details` 对象。
- MCP bridge 目标 TextContent 是适配器语义结果的紧凑阅读表达，不复制完整 JSON。
- CLI 默认阅读命令使用 `readable-view`；需要结构化消费或示例 JSON 校验时显式使用 `--output readable-json`；需要完整协议 envelope 时使用 `--output protocol-json` 或 adapter `invoke`。
- outline 是扁平 entries。
- find 是扁平 matches。
- ref 从 outline 原样交给 read。
- read 的 readable JSON 和 MCP bridge 目标 structuredContent 保留 `content_type`。
- 分页结果返回下一页 page；null 表示没有更多信息。

## Outline

| 边界 | 请求 | 响应 |
| --- | --- | --- |
| adapter invoke | [protocol-outline-request.json](json/protocol-outline-request.json) | [protocol-outline-response.json](json/protocol-outline-response.json) |
| MCP | [mcp-outline-request.json](json/mcp-outline-request.json) | [mcp-outline-response.json](json/mcp-outline-response.json) |
| readable JSON | 不适用 | [readable-outline.json](json/readable-outline.json) |

默认 CLI `readable-view` 输出由 pretty JSON header 承载相同 readable 字段；无 block 的 outline 不产生 block section。`readable-view` framing 的验收边界见 [输出模式](../output.md)，schema 校验范围见 [JSON Schema 索引](../schemas/json-schema.md)。

invoke 请求显式传入 `page: 1`、`limit_chars: 80` 和 `options.max_heading_level: 3`。结果返回 `page: 2`，表明还有更多条目且应继续请求第二页。

MCP bridge 目标 tool 的 `outputSchema` 内联 readable schema，见 [mcp-outline-tool.json](json/mcp-outline-tool.json)，并允许顶层 `warnings`。

## Ref 与 Read

outline 返回：

```text
H:L4:H2
```

该 ref 在 [protocol-read-request.json](json/protocol-read-request.json) 和 [mcp-read-request.json](json/mcp-read-request.json) 中原样复用。

| 边界 | 请求 | 响应 |
| --- | --- | --- |
| adapter invoke | [protocol-read-request.json](json/protocol-read-request.json) | [protocol-read-response.json](json/protocol-read-response.json) |
| MCP | [mcp-read-request.json](json/mcp-read-request.json) | [mcp-read-response.json](json/mcp-read-response.json) |
| readable JSON | 不适用 | [readable-read.json](json/readable-read.json) |

默认 CLI `readable-view` read 输出将 `/content` 外置为 block，header 中保留 `ref`、`content_type`、`cost` 和 `page`。需要直接解析 `content` 字符串的示例和工具应使用 `--output readable-json`。

read 使用 `page: 1` 和 `limit_chars: 64`，因此结果返回 `page: 2`；结果保留 `content_type: text/markdown`。保持 path、ref 和 limit_chars 不变并请求第二页即可继续读取。

MCP bridge 目标 tool 定义见 [mcp-read-tool.json](json/mcp-read-tool.json)。

## 发现与错误

- [manifest.json](json/manifest.json)
- [probe-result.json](json/probe-result.json)
- [error-ref-ambiguous.json](json/error-ref-ambiguous.json)
- [error-ref-invalid.json](json/error-ref-invalid.json)
- [error-format-unknown.json](json/error-format-unknown.json)
- [error-format-ambiguous.json](json/error-format-ambiguous.json)
- [error-invalid-request.json](json/error-invalid-request.json)

`find` 与 `info` 能力示例：

- [protocol-find-request.json](json/protocol-find-request.json) / [response](json/protocol-find-response.json) / [readable](json/readable-find.json)
- [protocol-info-request.json](json/protocol-info-request.json) / [response](json/protocol-info-response.json) / [readable](json/readable-info.json)
- [mcp-find-request.json](json/mcp-find-request.json) / [response](json/mcp-find-response.json) / [tool](json/mcp-find-tool.json)
- [mcp-info-request.json](json/mcp-info-request.json) / [response](json/mcp-info-response.json) / [tool](json/mcp-info-tool.json)

## 配置示例

- [docnav-markdown-config.json](json/docnav-markdown-config.json) 展示 `docnav-markdown` JSON 配置文件的文档化字段，对应 [docnav-markdown-config.schema.json](../schemas/docnav-markdown-config.schema.json)。

配置示例只证明文件形状和示例值。配置发现、字段映射、来源合并、失败边界和字段语义由 [标准参数](../standard-parameters.md)、[适配器契约](../adapter-contract.md) 和 [Markdown Adapter](../adapters/markdown.md) 拥有。

## Schema

原始协议和阅读输出由不同 schema 校验，见 [JSON Schema 索引](../schemas/json-schema.md)。

示例只证明 protocol/readable、manifest、probe 和配置文件示例的 documented shape。direct CLI warning placement、protocol-json stdout purity、adapter machine command 边界、配置读取行为和 pagination mechanics 由主规范、smoke 和 Rust 测试共同证明。
