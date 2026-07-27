# Claim CLAIM-BB-CORE-OUTPUT-001: Core 文档输出模式不混层

Topic: `core-cli`
Owner ref: `docs/output.md#输出层边界`

Statement:
- Readable document output and protocol JSON preserve one business result without mixing their transport envelopes.

Observations:
- 省略 output 和显式 `readable-view` 产生相同的 built-in readable-view text contract；`protocol-json` 对同一文档结果输出完整 protocol envelope。
- Readable text 与 protocol JSON 保持隔离：presentation-only display/cost/framing 不进入 protocol raw result，success/failure 仍分别保留当前 owner 的可观察语义。
- CLI 显式选择或 project config 选择 `protocol-json` 时，navigation response 产生前的 document failure 仍输出完整 failure envelope，不回退到 `readable-view`；project-selected 分支同时覆盖低优先级 user config 加载失败。
- CLI 或 config 使用已删除的 `readable-json` 时走普通 invalid-value boundary，不产生 alias、fallback 或 document output。
- Unstructured outline selected by config 在 `readable-view` 中作为 content block、在 `protocol-json` 中作为 raw result 可观察，并且不虚构 entries/ref/page/continuation。

Supported by:
- `smoke|core:document-output-boundary|CORE-OUTPUT-001`
