# JSON Schema 索引

本目录使用 JSON Schema Draft 2020-12，并按语义层拆分。Schema 是字段形状和示例的校验材料，不是新的规范来源；字段语义、输出承载和职责边界由对应主规范文档定义。

## 原始协议层

| Schema | 用途 |
| --- | --- |
| [protocol-request.schema.json](protocol-request.schema.json) | adapter invoke 请求 |
| [protocol-response.schema.json](protocol-response.schema.json) | adapter invoke 和 CLI `--output protocol-json` |
| [manifest.schema.json](manifest.schema.json) | adapter manifest |
| [probe-result.schema.json](probe-result.schema.json) | adapter probe |

## 阅读输出层

| Schema | 用途 |
| --- | --- |
| [readable-outline.schema.json](readable-outline.schema.json) | CLI `readable-json`、MCP outline structuredContent |
| [readable-read.schema.json](readable-read.schema.json) | CLI `readable-json`、MCP read structuredContent |
| [readable-find.schema.json](readable-find.schema.json) | CLI `readable-json`、MCP find structuredContent |
| [readable-info.schema.json](readable-info.schema.json) | CLI `readable-json`、MCP info structuredContent |
| [readable-error.schema.json](readable-error.schema.json) | CLI/MCP 精简错误 |
| [readable-common.schema.json](readable-common.schema.json) | readable/MCP schema 共享 `$defs` |

`readable-view` 和 `readable-json` 从同一 typed readable payload 派生。readable schema 只校验 CLI `readable-json` 和 MCP structuredContent。`readable-view` 不使用 readable JSON schema 校验；framing、header block refs 和 payload 还原的验收边界见 [输出模式](../output.md) 和 readable-view conformance vectors。protocol schema 保持独立。

原始协议和阅读输出不得互相使用对方 schema。`protocol-response.schema.json` 使用响应 `operation` 校验成功 result 类型，并从 [error-rules.json](../protocol/error-rules.json) 生成稳定错误 required details 校验块；稳定错误语义仍由 [原始协议](../protocol.md) 拥有。原始协议 schema 是机器稳定接口校验；阅读输出 schema 用于文档示例、MCP tool 声明和实现自测，不表示 readable 输出是长期机器解析协议。

operation readable schema 和 MCP structuredContent outputSchema 包含可省略的顶层 `warnings` 数组。warning 的来源、承载位置和稳定字段由 [输出模式](../output.md) 与 [MCP Handoff](../mcp.md) 定义；本文件只维护 schema `$defs` 与示例校验入口。

`readable-common.schema.json` 提供 readable/MCP 复用的 `capability`、`entry`、`page`、`warning` 和 `warnings` 定义。operation readable schema 可通过同目录 `$ref` 复用这些定义；发布 MCP tool `outputSchema` 时仍必须内联或随工具声明打包，不能要求 client 远程解析 schema URL。

本仓库的 docs validator 和 Markdown smoke 会先预加载 `docs/schemas/` 下的 schema，再按 `$id` 编译入口 schema；新增跨文件 `$ref` 时，应保持同目录相对引用，并为被引用 schema 设置稳定 `$id`。示例语义一致性由文档验证脚本检查，检查项必须能追溯到对应 owner 文档。

文件系统边界、ref 唯一性、真实分页一致性和配置优先级不属于 JSON Schema 校验范围，应由对应 owner 文档下的实现级业务测试覆盖。

`docnav-json-io` 拥有低层 serialization、newline writing 和 write failure plumbing。protocol request/response、manifest、probe 和 readable schema 的字段 shape 仍由本目录维护；语义校验、错误归属和通道承载由对应 owner 文档与实现测试验收。

`$id` 中的 URL 是 schema 标识，不要求运行时联网访问。MCP tool 声明中的 `outputSchema` 必须内联或随工具声明打包，不依赖远程 schema URL，避免 client 无法解析外部 schema。
