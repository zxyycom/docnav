## Why

Docnav v0 的协议目标是单一当前契约：`docnav` 和 adapter 按当前 schema 与语义校验交互；manifest、probe 和 invoke 输出全部通过时可用，任一环节不一致时当前阶段失败。兼容边界是显式的：本 change 不设计协议版本协商、兼容迁移或多版本 adapter 共存机制；未来出现真实多版本需求时以独立 change 处理。

同时，manifest 的职责收敛为 adapter 能力声明。manifest 只声明 adapter 身份、支持格式、扩展名、content type 和 capabilities；`docnav` 只处理 core 通用参数；格式专属默认值由 adapter 自己的 CLI 和配置域拥有。

## What Changes

- **BREAKING**: 移除 adapter 选择和 manifest 校验中的协议版本区间协商语义。
- 保留 `protocol_version: "0.1"`、`manifest_version`、`probe_version` 作为当前 schema 识别字段；这些字段不承载兼容或迁移语义。
- 将 adapter 可用性定义为当前契约校验：manifest/probe/invoke 输出必须通过当前 schema、必需字段、字段类型、operation/result shape 和语义校验；不一致直接失败。
- 从 manifest 契约中移除协议范围字段 `protocol.min/max` 和 `recommended_parameters`。
- 将 manifest 收敛为 adapter 身份、支持格式、扩展名、content type 和 capabilities。
- 移除 `PROTOCOL_INCOMPATIBLE` 稳定错误语义；字段或语义不一致由当前校验阶段的稳定错误或候选失败证据表达。
- 同步协议文档、adapter 契约、schema、示例、`docnav-protocol` 类型和相关测试。

## Capabilities

### New Capabilities

- 无。

### Modified Capabilities

- `v0-contract-documentation`: 文档契约从协议版本范围兼容改为当前契约硬校验，并移除 manifest 推荐参数语义。
- `protocol-and-adapter-sdk-implementation`: 共享协议类型、schema、错误规则和 SDK 校验移除协议版本区间协商与 manifest 推荐参数字段。
- `markdown-adapter-v0-implementation`: Markdown adapter manifest 和测试不再声明协议范围或推荐参数。

## Impact

- 影响主规范：`docs/protocol.md`、`docs/adapter-contract.md`、`docs/architecture.md`、`docs/cli.md`、`docs/testing.md`。
- 影响 schema 和示例：manifest schema、protocol response/readable error schema、error examples、manifest example。
- 影响 Rust crates：`docnav-protocol` 的 manifest/version/error 类型、生成的 error rules、相关单元测试，以及依赖这些类型的 adapter SDK 和 Markdown adapter manifest 构造。
- 不改变 `protocol-json` envelope、operation/result shape、page、ref、content_type 或 readable 输出分层。
