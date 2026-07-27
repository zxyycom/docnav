# Claim CLAIM-BB-CORE-SOURCE-001: Core adapter source 来自 static registry

Topic: `core-cli`
Owner ref: `docs/adapter-contract.md#内置-adapter-接口`

Statement:
- Core release dispatches built-in adapters from its linked static registry.

Observations:
- core release 内置 adapter dispatch 使用 static registry 中的 linked adapter implementation。
- 默认 document operation 的 implementation source 与项目配置中的普通文件内容解耦。

Supported by:
- `smoke|core:registry-contract-failures|CORE-SOURCE-001`
