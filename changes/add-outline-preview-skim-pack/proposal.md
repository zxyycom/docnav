# Proposal

本计划为 outline 增加一个预算内、确定性的 skim preview，使首屏同时提供结构和少量正文样本，同时保持 adapter operation、ref 和基础 read/pagination 契约不变。

## Why

普通 outline 只给结构，调用方常需逐个 read 才能判断章节是否值得继续。一个由 core 编排、低智能且总预算受控的 preview 可以减少盲目往返，而不要求 adapter 学习跨章节策略或引入摘要模型。

## Outcome

显式 outline preview surface 在总预算内选择少量可读 entries，通过现有 read pipeline 取得样本，并在一个 typed `ProtocolResponse::Success` 中稳定表达 base outline、preview content、状态和 continuation；`protocol-json` 与内置 `readable-view` 从同一结果投影。

## Scope

- Core 按 outline 顺序、非空 ref、preview count 和总预算做确定性选择，再复用现有 read pipeline。
- Preview result 表达 success、skipped、pending、read diagnostic 和 continuation；单个 preview 失败不升级为 outline primary failure。
- 不新增 adapter operation，不生成摘要、不智能排序、不推断用户意图，也不改变基础 `OutlineResult`、`ReadResult`、ref 或分页语义。
- 显式 CLI surface、count 和默认预算在实施首个 contract gate 中定稿。

## Success Criteria

- CLI/output owner 明确 preview surface、selection inputs、预算和两种输出模式的可观察契约。
- 同一 typed composition result 驱动 machine/readable output，不存在 renderer-only preview facts。
- 预算耗尽、无 ref、read diagnostic、分页和 continuation 都有稳定结果与测试。
- 普通 outline/read、所有 adapter contract 和 opaque ref handoff 保持兼容，workspace verification 通过。

## Affected Owners

- [CLI](../../docs/cli.md)：显式 preview surface、参数和命令行为。
- [原始协议](../../docs/protocol.md)与[输出模式](../../docs/output.md)：typed composition facts 及 raw/readable mapping。
- Core/navigation composition、shared protocol/output types、schema/examples、integration tests 与 Semantic Cases。
- Adapter 与 [Ref](../../docs/ref-contract.md) 只作为必须保持不变的边界。
