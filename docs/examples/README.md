# 端到端示例

本目录使用 [nested-duplicate.md](markdown/nested-duplicate.md) 展示 Markdown `outline -> ref -> read`、page、原始协议与阅读输出的边界。

## 关键原则

- `protocol-json` 返回包含 operation 的完整 protocol envelope。
- `docnav` 是识别格式、选择 adapter 和映射输出的核心 CLI。
- CLI 默认阅读命令使用 `readable-view`；需要稳定结构化消费或示例 JSON 校验时使用 `--output protocol-json`。
- protocol outline 是扁平 entries，entry 使用 `ref`、`label` 和可选结构化 facts。
- protocol find 是扁平 matches，match 使用同一 entry fact shape。
- ref 从 outline 原样交给 read。
- outline/find 当前返回结果中的非空 ref 去重后恰好只有一个，且追加 read 成功时，success result 增加 `auto_read`；其它 outcome 保持原 base result。
- protocol read 使用 `cost.measurements[]`。
- 分页结果返回下一页 page；null 表示没有更多信息。

## Outline

| 边界 | 请求 | 响应 |
| --- | --- | --- |
| protocol-json | [protocol-outline-request.json](json/protocol-outline-request.json) | [protocol-outline-response.json](json/protocol-outline-response.json) |

Structured outline 示例使用 `kind: "structured"`、entries 和 page。非结构化全文 outline 示例包括 path selector 触发的 [protocol-outline-unstructured-path-response.json](json/protocol-outline-unstructured-path-response.json)、cost threshold 触发的 [protocol-outline-unstructured-cost-response.json](json/protocol-outline-unstructured-cost-response.json)，以及 threshold 不命中后保持 structured 的 [protocol-outline-threshold-miss-response.json](json/protocol-outline-threshold-miss-response.json)。

[protocol-outline-auto-read-response.json](json/protocol-outline-auto-read-response.json) 展示当前返回结果只有一个 distinct ref 时成功追加完整 `ReadResult`。它同时保留 base `entries` 和非 null `page`，说明 auto-read 只判断当前返回结果，不改变 outline continuation。[protocol-outline-response.json](json/protocol-outline-response.json) 返回多个 distinct ref，因此保持无 `auto_read` 的 base result。

默认 CLI `readable-view` 由 built-in renderer 从同一个 protocol response 派生，并按 outline 的结果形态呈现内容：

- 不含 `auto_read` 的 structured result 只输出 base header，不产生 block；
- 含 successful `auto_read` 的 structured result 保留 base header，并用 `/auto_read/read/content` block 承载 nested read content；
- unstructured result 使用 `/content` block 承载全文。

完整的字段投影和 framing 契约见 [输出模式](../output.md)，schema 校验范围见 [JSON Schema 索引](../schemas/json-schema.md)。

protocol 请求显式传入 `page: 1`、`limit: 28` 和 request `arguments.options.max_heading_level: 3`。这是 operation request argument，不是持久 config source path；配置文件中的 Markdown native option 使用 `options.docnav-markdown.max_heading_level`。结果返回 `page: 2`，表明还有更多条目且应继续请求第二页。

## Ref 与 Read

outline 返回：

```text
H:L4:H2
```

该 ref 在 [protocol-read-request.json](json/protocol-read-request.json) 中原样复用。

| 边界 | 请求 | 响应 |
| --- | --- | --- |
| protocol-json | [protocol-read-request.json](json/protocol-read-request.json) | [protocol-read-response.json](json/protocol-read-response.json) |

默认 CLI `readable-view` read 输出将 `/content` 外置为 block，header 中保留 `ref`、`content_type`、由 `cost.measurements[]` 派生的 `cost` 摘要和 `page`。需要直接解析稳定结构化 `content` 字段的工具应使用 `--output protocol-json`。

read 使用 `page: 1` 和 `limit: 64`，因此结果返回 `page: 2`；结果保留 `content_type: text/markdown`。保持 path、ref 和 limit 不变并请求第二页即可继续读取。

## 发现与错误

- [manifest.json](json/manifest.json)
- [probe-result.json](json/probe-result.json)
- [error-ref-ambiguous.json](json/error-ref-ambiguous.json)
- [error-ref-invalid.json](json/error-ref-invalid.json)
- [error-ref-not-found.json](json/error-ref-not-found.json)
- [error-format-unknown.json](json/error-format-unknown.json)
- [error-format-ambiguous.json](json/error-format-ambiguous.json)
- [error-invalid-request.json](json/error-invalid-request.json)
- [error-unknown-argv.json](json/error-unknown-argv.json)
- [error-extra-positional.json](json/error-extra-positional.json)
- [error-operation-inapplicable-flag.json](json/error-operation-inapplicable-flag.json)
- [error-adapter-option-invalid.json](json/error-adapter-option-invalid.json)
- [error-explicit-adapter-failure.json](json/error-explicit-adapter-failure.json)
- [error-explicit-config-failure.json](json/error-explicit-config-failure.json)
- [error-unknown-config-field.json](json/error-unknown-config-field.json)

错误示例展示 protocol surface 投影。Protocol 错误 code、canonical details 和 primary diagnostic 字段由 [原始协议](../protocol.md#协议错误对象) 拥有；readable-view failure presentation 由 [输出模式](../output.md) 拥有。本目录不作为 code/details 规则来源。

`find` 与 `info` 能力示例：

- [protocol-find-request.json](json/protocol-find-request.json) / [response](json/protocol-find-response.json)
- [protocol-find-auto-read-response.json](json/protocol-find-auto-read-response.json) 展示多个 match 共享同一 opaque ref 时只附加一个成功 `auto_read`
- [protocol-info-request.json](json/protocol-info-request.json) / [response](json/protocol-info-response.json)

## 配置示例

- [docnav-markdown-config.json](json/docnav-markdown-config.json) 展示 `docnav` 配置 source 中 `options.docnav-markdown.max_heading_level` Markdown native option、document operation defaults 和 core-owned `invocation_log` section 的文档化 shape，对应 [docnav-markdown-config.schema.json](../schemas/docnav-markdown-config.schema.json)。Project 和 user config 使用同一个 `defaults.auto_read` JSON locator；其 canonical catalog identity 是 `docnav.defaults.auto_read`，`docnav config inspect` 只读展示对应 source candidate。
- [docnav-markdown-config-path-unstructured.json](json/docnav-markdown-config-path-unstructured.json) 展示 `outline.mode_rules[]` path selector 和后写 rule 覆盖。
- [docnav-markdown-config-cost-unstructured.json](json/docnav-markdown-config-cost-unstructured.json) 展示 adapter-scoped cost threshold selector。
- [docnav-markdown-config-threshold-miss.json](json/docnav-markdown-config-threshold-miss.json) 展示 threshold candidate filtering 和不命中时保持 structured 的输入形状。
- [error-invalid-path-rule.json](json/error-invalid-path-rule.json) 展示 invalid path rule 的 protocol error projection。

配置示例只证明文件形状和示例值。配置发现、字段映射、来源合并、selector priority、matcher diagnostics、threshold comparison、失败边界和字段语义由 [Navigation Input Resolution](../navigation-input-resolution.md)、[适配器契约](../adapter-contract.md) 和 [Markdown Adapter](../adapters/markdown.md) 拥有。

## Runtime invocation log

- [invocation-log-operation-completed.json](json/invocation-log-operation-completed.json) 展示 metadata-only successful operation event：正文通过 `hash_algorithm: "sha256"`、小写 64 位十六进制 `content_hash`、content type 和 size metadata 引用，不 inline 完整 content。
- [invocation-log-operation-failed.json](json/invocation-log-operation-failed.json) 展示 bounded failure event：记录 failure layer、stable code when available 和 bounded summary。
- [invocation-log-content-captured.json](json/invocation-log-content-captured.json) 展示 content capture event：正文位置只通过 `relative_path` 表达，格式为 `<YYYY-MM-DD>/sha256-<content_hash>.content`。
- [invocation-log-content-capture-failed.json](json/invocation-log-content-capture-failed.json) 展示 content capture failure event：失败诊断与正文 hash、content type 和 size metadata 分开记录，不 inline 完整 content。

Invocation log 示例只证明 [invocation-log-event.schema.json](../schemas/invocation-log-event.schema.json) 的 documented shape。CLI/config 开关、sink/path、content capture root、日志失败降级、stdout purity 和 hash 计算输入由对应主规范和实现测试证明。

## Schema

原始协议由 protocol schema 校验，见 [JSON Schema 索引](../schemas/json-schema.md)。Protocol 示例证明 raw 结构化字段；readable-view 的最终文本由 built-in renderer conformance 承接，不发布另一组 JSON examples。

示例只证明 protocol、manifest、probe、配置文件示例和 invocation log event 的 documented shape 与投影结果。Core CLI strict failure、primary diagnostic projection、protocol-json stdout purity、readable-view presentation、adapter inspection 边界、配置读取行为、invocation logging side-effect 边界和 pagination mechanics 由主规范、smoke 和 Rust 测试共同证明。
