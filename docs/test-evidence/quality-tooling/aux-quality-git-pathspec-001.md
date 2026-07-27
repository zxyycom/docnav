### Case AUX-QUALITY-GIT-PATHSPEC-001: Quality git pathspec 参数稳定

Entry:
- `scripts/tools/quality-core/src/input/files.test.ts > quality input git pathspecs > builds explicit git pathspec arguments and can omit empty pathspecs`

Contract:
- `docs/tooling.md` 定义或约束“Quality git pathspec 参数稳定”所涉及的稳定行为边界。

Proves:
- quality input git pathspec 参数使用显式 `--` 分隔并保留 glob pathspec magic。
- 空 pathspec 可按调用方需要保留 `--` 或完全省略。
