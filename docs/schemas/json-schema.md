# JSON Schema 索引

本目录使用 JSON Schema Draft 2020-12，并按输出类型拆分。Schema 是字段形状、surface 投影和示例的校验材料，不是新的规范来源；字段语义、诊断 code/details 规则、输出承载和职责边界由对应主规范文档定义。

## 原始协议

| Schema | 用途 |
| --- | --- |
| [protocol-request.schema.json](protocol-request.schema.json) | protocol-json request envelope |
| [protocol-response.schema.json](protocol-response.schema.json) | CLI `--output protocol-json` response envelope |
| [manifest.schema.json](manifest.schema.json) | adapter manifest |
| [probe-result.schema.json](probe-result.schema.json) | adapter probe |

## 阅读输出

| Schema | 用途 |
| --- | --- |
| [readable-outline.schema.json](readable-outline.schema.json) | CLI `readable-json` outline |
| [readable-read.schema.json](readable-read.schema.json) | CLI `readable-json` read |
| [readable-find.schema.json](readable-find.schema.json) | CLI `readable-json` find |
| [readable-info.schema.json](readable-info.schema.json) | CLI `readable-json` info |
| [readable-error.schema.json](readable-error.schema.json) | CLI 精简错误 |
| [readable-common.schema.json](readable-common.schema.json) | readable schema 共享 `$defs` |

`readable-view` 和 `readable-json` 从同一 typed readable payload 派生。readable schema 只校验 CLI `readable-json`。`readable-view` 不使用 readable JSON schema 校验；framing、header block refs 和 payload 还原的验收边界见 [输出模式](../output.md) 和 readable-view conformance vectors。protocol schema 保持独立。

原始协议和阅读输出不得互相使用对方 schema。`protocol-response.schema.json` 使用响应 `operation` 校验成功 result 类型，并消费 [错误通道](../diagnostics.md) 中 primary `DiagnosticRecord` 的 protocol 投影生成错误字段和 details 校验块；protocol envelope 和投影字段由 [原始协议](../protocol.md) 拥有。原始协议 schema 是机器稳定接口校验材料，用于示例、fixture、CI drift check 和第三方对齐；production runtime decode path 的字段级校验由 `docnav-typed-fields` contract validation 承接。阅读输出 schema 用于文档示例和实现自测，不表示 readable 输出是长期机器解析协议。

protocol response 示例应证明 outline/find item facts、read `cost.measurements[]` 和 info `document`/`adapter`/`metadata` 的 raw shape；readable 示例应证明由这些 facts 派生的 `display`、成本摘要和精简 info display。任一 schema 都不能接受对方层独有字段作为成功 result 的替代形态。

operation readable schema 只描述 successful document payload 和该 output mode 拥有的结构。Rejected public input、invalid config、explicit adapter/ref failure 和 automatic discovery all-failed candidate lists 由 [readable-error.schema.json](readable-error.schema.json) 校验 primary `DiagnosticRecord` readable projection；successful readable schema 不承载被拒绝输入或失败候选信息。

`readable-common.schema.json` 提供 readable 复用的 `capability`、`entry`、`page` 和 diagnostic projection 定义。operation readable schema 可通过同目录 `$ref` 复用这些定义。

## 配置参考层

| Schema | 用途 |
| --- | --- |
| [docnav-markdown-config.schema.json](docnav-markdown-config.schema.json) | Core `docnav` 配置中 Markdown native option 相关字段形状和示例校验；adapter-owned option range 语义不由 config schema 拥有 |

配置 schema 只描述文档化的 JSON 文件形状，可用于示例校验和编辑器提示。配置文件发现、字段映射、来源合并、strict unmapped field failure、错误归属和 adapter-specific 字段语义由 [标准参数](../standard-parameters.md)、[适配器契约](../adapter-contract.md) 和对应 adapter 文档拥有。

本仓库的 docs validator 和 core smoke 会先预加载 `docs/schemas/` 下的 schema，再按 `$id` 编译入口 schema；新增跨文件 `$ref` 时，应保持同目录相对引用，并为被引用 schema 设置稳定 `$id`。示例语义一致性由文档验证脚本检查，检查项必须能追溯到对应 owner 文档。

文件系统边界、ref 唯一性、真实分页一致性和配置优先级不属于 JSON Schema 校验范围，应由对应 owner 文档下的实现级业务测试覆盖。

`docnav-json-io` 拥有低层 serialization、newline writing 和 write failure plumbing。protocol request/response、manifest、probe 和 readable schema 的字段 shape 仍由本目录维护；语义校验、错误归属、诊断投影和通道承载由对应 owner 文档与实现测试验收。

`$id` 中的 URL 是 schema 标识，不要求运行时联网访问。
