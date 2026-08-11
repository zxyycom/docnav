# Proposal

本计划在一次完整人工决策门禁后重设计单文档 find result 的逻辑单元、字段、工作预算、分页、continuation、auto-read 和迁移边界，再作为一个垂直切片交付。

## Why

Current find 复用 occurrence-oriented `Entry`：Markdown 每个 source occurrence 产生一项，`label` 是命中片段，`location` 是命中行，同一 readable ref 可以重复。改成 distinct node 或 grouped result 会同时改变 identity、multiplicity、ordering、pagination、continuation、auto-read eligibility 和为形成一页允许扫描/保留多少 source；不能只替换字段 shape。

## Outcome

用户或指定 product/architecture owner 从 occurrence、distinct exact-ref/node 或 grouped 模型中批准一个完整 contract 与 work budget；获批 Target 只由本 Change 的 design Decisions 完整承接，跨 change 方向交给决策 owner。实施前只审计稳定 owner、schema 和 examples 相对 Target 的 delta；实现及行为验证成立后，才把 shared protocol/navigation、Markdown、必要的 JSON handoff、readable output、schema/examples 和 release materials 同步为 Current。批准及同步前，稳定 owner 继续描述 Current occurrence behavior。

## Scope

- 一次性决定 logical unit、Rust/wire type、top-level field、九个 Current `Entry` 字段的 disposition、identity、multiplicity、evidence、ordering、pagination、continuation 和 auto-read scope。
- 同时决定 first/later-page scan、retained state、lookahead/replay、budget exhaustion 和 compatibility/version/rollback。
- Model 对所有 adapter 是共享 contract；若需要格式变体，必须有显式 public discriminator。
- Token estimator/calculator 留给独立 token-cost change；JSON presentation 留给 JSON renderer change。
- 不在人工门禁前修改 owner、schema、测试预期或 production implementation。

## Success Criteria

- 完整 decision packet 获明确批准，所有候选依赖语言被替换为一个 exact Target，且长期方向在用户明确授权维护决策后按 `decision-records` 交接。
- Protocol/output/navigation/Markdown 及必要 JSON owner 对 logical unit、字段、ordering、page、continuation、auto-read 与 budgets 一致。
- 实现用可证明的单调 traversal/replay 和 bounded retained work 产生 deterministic pages，不用未批准的 full scan/index/spill。
- 兼容或 breaking migration、rollback、schema/examples、真实 CLI/package 和 Semantic Cases 全部有证据。

## Affected Owners

- [原始协议](../../docs/protocol.md)：find logical unit、wire fields、page/continuation 和 auto-read facts。
- [Navigation Input Resolution](../../docs/navigation-input-resolution.md)：request construction、dispatch、auto-read 和 bounded replay。
- [Markdown Adapter](../../docs/adapters/markdown.md)及必要时的 [JSON Adapter](../../docs/adapters/json.md)：format-specific production、ordering 与 evidence。
- [输出模式](../../docs/output.md)、schema/examples、shared types、tests、Semantic Cases 与 release validation。
- [将 token cost 作为有界性能债务修复](../../docs/decisions/product-direction/repair-token-cost-as-bounded-debt.md)只拥有 estimator 方向，不是本计划前置。
