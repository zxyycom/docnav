### Case BB-CORE-LINK-001: Core 原样传递真实 Markdown ref

Entry:
- `test/smoke/core/cases/real-markdown.ts > smoke task CORE-LINK-001`

Contract:
- `docs/ref-contract.md` 定义或约束“Core 原样传递真实 Markdown ref”所涉及的稳定行为边界。

Proves:
- 真实 `docnav` 进程可以通过 Markdown adapter 完成 `outline -> ref -> read`、`find -> ref -> read` 和 `info` 链路。
- outline/find 返回的 adapter ref 可原样提交给 read，`readable-view` read 保留该 ref；用户可见阅读文本不包含 protocol envelope。
