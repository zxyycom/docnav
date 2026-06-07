## 1. 文档契约同步

- [ ] 1.1 更新 `docs/protocol.md`，保留固定 version 识别字段和可选 `arguments.options` 字段，移除协议版本区间协商、最高兼容版本选择和 manifest 推荐参数来源语义。
- [ ] 1.2 更新 `docs/adapter-contract.md`，移除 manifest `protocol.min/max` 和 `recommended_parameters`，改为当前 schema 与语义硬校验。
- [ ] 1.3 更新 `docs/architecture.md`、`docs/cli.md` 和 `docs/testing.md` 中的 adapter 安装、更新、选择、默认值所有权和验证规则。
- [ ] 1.4 用局部搜索更新文档摘要中提到协议范围、`PROTOCOL_INCOMPATIBLE` 或 manifest 推荐参数的说明，至少覆盖 `docs/navigation.md` 和 `docs/examples/README.md` 的命中项。

## 2. Schema、示例和错误规则

- [ ] 2.1 更新 `docs/schemas/manifest.schema.json`，移除 manifest `protocol.min/max` 和 `recommended_parameters` 字段，并通过 `additionalProperties: false` 拒绝旧字段。
- [ ] 2.2 从 `docs/protocol/error-rules.json` 移除 `PROTOCOL_INCOMPATIBLE`，并重新生成 Rust error rules。
- [ ] 2.3 更新 `docs/schemas/protocol-response.schema.json` 和 `docs/schemas/readable-error.schema.json`，移除 `PROTOCOL_INCOMPATIBLE` 分支。
- [ ] 2.4 移除 `docs/examples/json/error-protocol-incompatible.json` 及其 README 引用。
- [ ] 2.5 更新 `docs/examples/json/manifest.json`，不再声明协议范围或推荐参数。

## 3. Rust 协议与 SDK 实现

- [ ] 3.1 从 `docnav-protocol` 公共类型中移除 `ProtocolRange`、协议闭区间选择 API、manifest protocol range 字段和 manifest recommended parameters 字段。
- [ ] 3.2 从 `StableErrorCode` 和相关构造函数中移除 `ProtocolIncompatible`。
- [ ] 3.3 更新 request/invoke 校验逻辑：`protocol_version` 只按当前 schema 字段校验，不做范围协商；无法解析请求或无法提取版本字段时，failure envelope 使用当前 `PROTOCOL_VERSION` 常量。
- [ ] 3.4 保留 invoke argument 类型中的可选 `options` 字段，但确保 `docnav-protocol`、adapter SDK 和测试不再把 manifest `recommended_parameters` 作为 options 来源。
- [ ] 3.5 更新 adapter SDK manifest、probe 和 invoke 边界测试，覆盖字段 shape 校验失败而不是版本范围失败。

## 4. Markdown Adapter 更新

- [ ] 4.1 更新 `docnav-markdown` manifest 构造，不再输出 `protocol.min/max` 或 `recommended_parameters`。
- [ ] 4.2 更新 Markdown adapter 和 CLI 测试中对 manifest protocol range 与 recommended parameters 的断言。
- [ ] 4.3 确认 Markdown parser、ref、pagination、operation handler 和 adapter 自有格式默认值行为不受影响。

## 5. 验证

- [ ] 5.1 运行 schema 和示例验证，确认 manifest、protocol response、readable error 示例全部通过。
- [ ] 5.2 运行 Rust workspace tests，确认协议类型、SDK 和 Markdown adapter 测试通过。
- [ ] 5.3 运行 `openspec validate simplify-protocol-compatibility-to-field-alignment --strict`。
- [ ] 5.4 完成跨文档、schema、示例和 Rust 的交付后运行 `pnpm run verify:docnav-workspace`，记录通过结果或阻塞命令输出。
