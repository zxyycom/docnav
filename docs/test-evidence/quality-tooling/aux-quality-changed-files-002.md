### Case AUX-QUALITY-CHANGED-FILES-002: Fails fast when an explicit changed files list cannot be read

Entry:
- `scripts/tools/quality-core/src/input/files.test.ts > quality changed file input > fails fast when an explicit changed-files list cannot be read`

Contract:
- `docs/tooling.md` 定义或约束“Quality revision inputs 保持 current/changed/baseline 一致”所涉及的稳定行为边界。

Proves:
- 原生入口 `quality changed file input > fails fast when an explicit changed-files list cannot be read` 直接验证“Fails fast when an explicit changed files list cannot be read”所描述的结果。
