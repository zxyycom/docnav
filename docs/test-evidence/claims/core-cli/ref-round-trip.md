# Claim CLAIM-CLI-REF-ROUND-TRIP-001: Core 原样传递真实 Markdown ref

Topic: `core-cli`
Owner ref: `docs/ref-contract.md#共享调用流程`

Statement:
- Core passes adapter-generated refs unchanged from outline or find into read.

Observations:
- 真实 `docnav` 进程可以通过 Markdown adapter 完成 `outline -> ref -> read`、`find -> ref -> read` 和 `info` 链路。
- outline/find 返回的 adapter ref 可原样提交给 read，`readable-view` read 保留该 ref；用户可见阅读文本不包含 protocol envelope。

Supported by:
- `smoke|core:real-markdown-link-chain|CORE-LINK-001`
