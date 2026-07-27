### Case AUX-QUALITY-FINGERPRINT-001: Quality input fingerprint 稳定

Entry:
- `scripts/tools/quality-core/src/input/files.test.ts > quality input fingerprints > uses stable SHA-256 fingerprints for sorted file content`

Contract:
- `docs/tooling.md` 定义或约束“Quality input fingerprint 稳定”所涉及的稳定行为边界。

Proves:
- quality input fingerprint 使用排序后的文件内容生成稳定 SHA-256。
- 文件内容变化会改变 fingerprint，文件顺序变化不会改变 fingerprint。
