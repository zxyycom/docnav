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

`readable-view` 是文本输出 contract，不发布独立 JSON Schema。其 header、block reference、framing、payload 还原和 error presentation 由 [输出模式](../output.md) 定义，并由 built-in renderer conformance vectors 验证。

`protocol-response.schema.json` 只校验 protocol envelope 和 typed operation result，不作为 readable-view header schema。它使用响应 `operation` 校验成功 result 类型，并按 [原始协议](../protocol.md#协议错误对象) 中的 primary diagnostic projection 校验错误字段和 details；protocol envelope 和投影字段由 [原始协议](../protocol.md) 拥有。原始协议 schema 是机器稳定接口校验材料，用于示例、fixture、CI drift check 和第三方对齐；production runtime decode path 的字段级校验由 `docnav-typed-fields` contract validation 承接。

protocol response 示例证明 outline/find item facts、read `cost.measurements[]` 和 info `document`/`adapter`/`metadata` 的 raw shape。Outline/find 的 `auto_read` 是 default-on orchestration 成功时才出现的 optional closed object：`reason` 固定为 `unique_ref`，`read` 复用既有 `ReadResult` schema。示例分别覆盖当前返回结果只有一个 distinct ref、多个 find match 共享同一 ref，以及多个 distinct ref 时保持 base result；base operation fields 和 base/read `page` continuation 保持原位置与含义。Readable-view presentation 从同一个 protocol response 派生，但不以另一组 JSON examples 或 schema 形成机器 contract。

## 配置参考层

| Schema | 用途 |
| --- | --- |
| [docnav-markdown-config.schema.json](docnav-markdown-config.schema.json) | `docnav` 配置 source 中 `defaults.auto_read`、`options.docnav-markdown.max_heading_level` Markdown native option、其它 document operation defaults、navigation-owned outline selector 和 core-owned `invocation_log` section 的字段形状和示例校验；core catalog 与 selected-operation view 的参数语义、selector priority、path matching、threshold comparison 和 invocation logging enablement 不由 config schema 拥有 |

配置 schema 只描述文档化的 JSON 文件形状，可用于示例校验和编辑器提示。配置文件发现、字段映射、来源合并、strict unmapped field failure、错误归属和 adapter-specific 字段语义由 [Navigation Input Resolution](../navigation-input-resolution.md)、[适配器契约](../adapter-contract.md) 和对应 adapter 文档拥有。

本仓库的 docs validator 和 core smoke 会先预加载 `docs/schemas/` 下的 schema，再按 `$id` 编译入口 schema；新增跨文件 `$ref` 时，应保持同目录相对引用，并为被引用 schema 设置稳定 `$id`。示例语义一致性由文档验证脚本检查，检查项必须能追溯到对应 owner 文档。

文件系统边界、ref 唯一性、真实分页一致性和配置优先级不属于 JSON Schema 校验范围，应由对应 owner 文档下的实现级业务测试覆盖。

`docnav-json-io` 拥有低层 serialization、newline writing 和 write failure plumbing。protocol request/response、manifest 和 probe 的字段 shape 仍由本目录维护；语义校验、错误归属、诊断投影和通道承载由对应 owner 文档与实现测试验收。

## Runtime invocation log

| Schema | 用途 |
| --- | --- |
| [invocation-log-event.schema.json](invocation-log-event.schema.json) | runtime invocation JSONL 单行 event validation material，覆盖 metadata-only operation success/failure event、content hash reference 和 content capture success/failure event variants |

Invocation log schema 只校验每一行 JSON event 的字段 shape。CLI/config 显式启用语义、sink/path、content capture root、路径规范化、日志写入失败降级、stdout/stderr placement、protocol envelope 和 adapter behavior 分别由 [CLI](../cli.md)、[架构](../architecture.md)、[输出模式](../output.md)、[原始协议](../protocol.md) 和 [Navigation Input Resolution](../navigation-input-resolution.md) 拥有。

`invocation-log-event.schema.json` 固定 `hash_algorithm: "sha256"`、小写 64 位十六进制 `content_hash`、content reference 的 content type 和 size metadata，以及 content capture `relative_path` 的 `<YYYY-MM-DD>/sha256-<content_hash>.content` shape。正文文件 bytes、hash 计算输入和 capture root 解析不是 schema 责任，必须由实现测试证明。

`$id` 中的 URL 是 schema 标识，不要求运行时联网访问。
