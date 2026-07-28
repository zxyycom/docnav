# Claim CLAIM-CLI-UNIQUE-REF-AUTO-READ-001: Core unique-ref auto-read 默认值与关闭来源可观察

Topic: `core-cli`
Owner ref: `docs/navigation-input-resolution.md#unique-ref-auto-read-composition`

Statement:
- A successful find with one distinct ref defaults to unique-ref auto-read unless an explicit source disables it.

Observations:
- 真实 `find` CLI 在所有 auto-read 来源省略且当前返回结果只有一个 distinct ref 时，默认以 `unique-ref` 追加 nested read；`protocol-json` 与 `readable-view` 从同一结果保留 ref、content type 和 nested content。
- CLI、默认 project config fixture 或显式 user config fixture 解析为 `disabled` 时，真实进程以退出码 `0` 返回原 base find，stdout 保持所选输出模式且 stderr 为空；代表性 protocol/readable 分支均不出现 `auto_read`，readable base projection 也不产生 block。

Supported by:
- `smoke|core:auto-read|CORE-AUTO-READ-001`
