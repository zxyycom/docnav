# JSON Schema

本目录使用 JSON Schema Draft 2020-12，并按语义层拆分。Schema 是字段形状和示例的校验材料，不是新的规范来源；字段语义和职责边界由对应主规范文档定义。

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
| [readable-outline.schema.json](readable-outline.schema.json) | CLI readable JSON、MCP outline structuredContent |
| [readable-read.schema.json](readable-read.schema.json) | CLI readable JSON、MCP read structuredContent |
| [readable-find.schema.json](readable-find.schema.json) | CLI readable JSON、MCP find structuredContent |
| [readable-info.schema.json](readable-info.schema.json) | CLI readable JSON、MCP info structuredContent |
| [readable-error.schema.json](readable-error.schema.json) | CLI/MCP 精简错误 |
| [readable-common.schema.json](readable-common.schema.json) | readable/MCP schema 共享 `$defs` |

原始协议和阅读输出不得互相使用对方 schema。`protocol-response.schema.json` 使用响应 `operation` 校验成功 result 类型，并从 [error-rules.json](../protocol/error-rules.json) 生成稳定错误 required details 校验块；稳定错误语义仍由 [原始协议](../protocol.md) 拥有。原始协议 schema 是机器稳定接口校验；阅读输出 schema 用于文档示例、MCP tool 声明和实现自测，不表示 readable 输出是长期机器解析协议。

operation readable schema 和 MCP structuredContent outputSchema 包含可省略的顶层 `warnings` 数组，用于承载直接 CLI argv 兼容性 warning 和 adapter 选择候选 warning；有 warning 的 payload 必须填充该字段。每个 warning item 使用稳定 envelope：`id`、非空 `reason`、稳定 `effect` 和 `details` 对象。当前稳定 family marker 包括 `cli_argv_ignored` 和 `adapter_candidate_failure`；CLI argv token 只通过 `details.tokens` 等 family-specific 字段表达，adapter candidate warning 在 `details` 中保留 `adapter_id`、`stage`、`code` 和可选 `preselected`。protocol response、manifest 和 probe schema 不包含 CLI warning 字段；这些机器输出存在 CLI warning 时由 stderr 承载。

`readable-common.schema.json` 提供 readable/MCP 复用的 `capability`、`entry`、`page`、`warning` 和 `warnings` 定义。operation readable schema 可通过同目录 `$ref` 复用这些定义；发布 MCP tool `outputSchema` 时仍必须内联或随工具声明打包，不能要求 client 远程解析 schema URL。

本仓库的 docs validator 和 Markdown smoke 会先预加载 `docs/schemas/` 下的 schema，再按 `$id` 编译入口 schema；新增跨文件 `$ref` 时，应保持同目录相对引用，并为被引用 schema 设置稳定 `$id`。

`scripts/validate-docs.mjs semantics` 对文档示例执行补充语义校验：协议 request/response 的 `protocol_version`、`request_id` 和 `operation` 必须配对；非 null page 必须是请求 page 加 1；示例阅读负载必须符合 `limit_chars`；protocol result、readable JSON 和 MCP structuredContent 必须保持同一业务语义；错误示例必须包含协议表声明的 required details 字段；Markdown manifest 示例必须声明 `outline`、`read`、`find` 和 `info` 全部能力。

文件系统边界、ref 唯一性、真实分页一致性和配置优先级仍需后续实现级业务测试。

`$id` 中的 URL 是 schema 标识，不要求运行时联网访问。MCP tool 声明中的 `outputSchema` 必须内联或随工具声明打包，不依赖远程 schema URL，避免 client 无法解析外部 schema。
