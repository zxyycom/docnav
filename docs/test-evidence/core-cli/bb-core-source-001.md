### Case BB-CORE-SOURCE-001: Core adapter source 来自 static registry

Entry:
- `test/smoke/core/cases/failures.ts > smoke task CORE-SOURCE-001`

Contract:
- `docs/adapter-contract.md` 定义或约束“Core adapter source 来自 static registry”所涉及的稳定行为边界。

Proves:
- core release 内置 adapter dispatch 使用 static registry 中的 linked adapter implementation。
- 默认 document operation 的 implementation source 与项目配置中的普通文件内容解耦。
