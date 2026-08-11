# Proposal

本计划让省略 path 的显式 `docnav find` 在 current project 内执行有限、确定性、可继续的跨文档搜索，同时保持显式 path 的单文档 find 契约。

## Why

调用方目前必须自行发现项目文件、复制 adapter selection、合并跨格式结果、组合 document path 与 opaque ref，并处理局部失败和分页。这重复 core/navigation 责任，也容易破坏单文档结果身份和 `outline -> ref -> read` 的可继续性。

## Outcome

`docnav find --query <text>` 在 resolved current project root 内惰性发现可支持文档，按稳定顺序逐文档复用 Current pathname routing 与 adapter find，把每个最终单文档 logical unit 包装为 `document.path + opaque ref` 的 project result，并以有限 numeric continuation 和局部失败 facts 支持继续读取；显式 path 行为保持不变。

## Scope

- Path 是否存在是 CLI scope discriminator；显式目录不作为 project alias。
- Navigation 按 per-directory sorted deterministic DFS 惰性遍历；只使用 project-owned ignore sources，普通 hidden entry 不被默认排除，不跟随 symlink、不按文件大小跳过、不预收集完整项目。
- Automatic/explicit routing 复用 Current pathname-hint → exact adapter seam；adapter 仍只处理一个文档。
- Project result 包装最终获批的单文档 find unit，不自行选择 occurrence/distinct/group 模型；project mode 不 auto-read。
- 局部文件/adapter failure 是 bounded success facts，project root、identity 或遍历基础失败仍为顶层 failure。
- 不增加持久索引、后台 daemon、跨运行 cache、ranking/query language、project-aware adapter operation 或 opaque cursor。

## Success Criteria

- `redesign-find-result-model` 已形成并实现稳定单文档 unit，project wrapper 不复制或改变其 identity、ordering、page、auto-read seam 和 evidence。
- CLI、raw request、project response、numeric replay、failure taxonomy 和 readable projection 在 owner、schema、types、examples 与 tests 中一致。
- 获批 traversal path 满足 ignore/symlink/order/identity/平台边界，private work quantum 保证有限工作和确定性推进且不泄漏为 public contract。
- Explicit-path find 的 argv、request/result fixture 和 output 完全兼容；project mode 的跨文档同 ref、局部失败、failure-only/empty continuation 和 terminal page 可重放。
- CLI/package smoke、Semantic Cases、schema/examples 和完整 workspace verification 通过。

## Affected Owners

- [CLI](../../docs/cli.md)：optional path、current project scope、argv/help 和退出行为。
- [Navigation Input Resolution](../../docs/navigation-input-resolution.md)：project discovery、routing orchestration、逐文档 dispatch 和 replay。
- [原始协议](../../docs/protocol.md)与[输出模式](../../docs/output.md)：closed request union、project result/page/failure 和 readable projection。
- `docs/schemas/`、`docs/examples/`、core/navigation/protocol/output implementation、tests、Semantic Cases 与 release validation。
- [适配器契约](../../docs/adapter-contract.md)与 [Ref](../../docs/ref-contract.md)：只作为必须保持的单文档 dispatch、limit 和 opaque ref 边界。
- [测试策略](../../docs/testing.md)与对应 Semantic Cases；`redesign-find-result-model` 提供单文档 logical unit。
