# Tasks

先关闭单文档 handoff、traversal dependency 和 private quantum 门禁，再同步 contract、实现 discovery/replay，最后证明跨文档身份、失败和 continuation。

## Readiness

- [x] 0.1 Proposal、design 和 tasks 共享 optional path → current project find 的单一目标，明确保持 explicit-path contract。
- [x] 0.2 CLI、navigation、protocol/output、schema/example 与 adapter/ref owner 边界完整。
- [x] 0.3 Current routing seam 已从 owner 恢复；单文档 find、dependency 和 quantum 是有 owner、关闭动作和被阻塞任务的前置门禁。
- [x] 0.4 Project identity、traversal、failure、replay、no-auto-read 和非目标已经形成可验证设计，没有隐藏跨运行状态。

## Implementation

- [ ] 1.1 从当前 owner、schema、Rust types 和 release evidence 重核 single-document find、project root、pathname routing、adapter dispatch 与 output 基线。
- [ ] 1.2 等待 `redesign-find-result-model` 完成最终 contract、实现和验证；从届时 Current owner/types/schema 取得 exact nested unit、ordering、continuation 和 auto-read seam，并重写本计划全部 overlapping assumptions，不修改 predecessor artifacts。
- [ ] 1.3 形成 dependency audit，比较至少一个成熟 ignore-aware walker、可行 alternatives 与 no-new-dependency；证据覆盖维护/采用度、安全 advisories/unsafe/transitives、license/notice、MSRV/toolchain/targets、features、package size、cold/warm startup、directory fanout、ignore/symlink/order/identity correctness 和 rollback。
- [ ] 1.4 由用户或指定 architecture owner 批准 exact traversal crate/version/features 或 no-new-dependency；批准前不得修改 Cargo manifest、lockfile 或 production traversal。
- [ ] 1.5 用 empty/filter-heavy/multi-page/local-failure workloads 选择正数、有限、same-build fixed 的 private quantum，证明每个 non-fatal transition 推进、page/wrapper 有界、empty continuation 可达、stable replay deterministic 且无 busy loop；exact value 不写入 public artifacts。
- [ ] 1.6 将 1.2–1.5 的结果写入 design Decisions，清空 Open Questions，并复核 request/result、routing/filtering、failure 和 replay contract 完整；门禁未全部关闭时不得开始 2.1 及之后任务。
- [ ] 2.1 保持 design Decisions 为唯一 change-local Target，逐项审计未来需要同步到 CLI、navigation、protocol 和 output 稳定 owner 的 Current delta；将 optional path、project context、discovery/routing/replay、closed request/result/failure、process mapping、readable mapping 的目标位置和成立所需证据登记回本 design，不在证据闭合前修改稳定 owner。
- [ ] 2.2 更新 request/response schema、runtime decode/validation 和 examples，保留 existing single-document encoding，新增 project root、multi-document matches、mixed/failure-only/empty-continuable result、local failures 和 terminal page。
- [ ] 2.3 在修改测试前依次读取 `docs/testing.md`、行为 owner、`docs/testing/case-maintenance.md` 和 `test-evidence-review` skill，并运行 `bun run test-evidence -- check --root .` 证明当前 static/runtime/Case 映射闭合。
- [ ] 2.4 先建立失败测试与独立 fixtures，覆盖 closed request union、旧 fixture compatibility、project result validation、raw/readable parity、path identity、failure taxonomy、work quantum 和 numeric replay。
- [ ] 3.1 修改 CLI argv/help/preflight，使 supplied file、omitted path、explicit directory、extra positional、missing query 和 project-inapplicable auto-read 进入确定 branch。
- [ ] 3.2 按批准结果更新依赖或保持 no-new-dependency，并实现 lazy per-directory sorted DFS、project-owned ignore、control-dir/symlink skip、ordinary hidden inclusion、lossless identity 和 fatal/local diagnostics；不得 flat collect、follow symlink 或 silent large-file skip。
- [ ] 3.3 复用 Current automatic complete-basename/exact-format routing 与 explicit exact-id seam，逐文档构造 selected view 并调用既有 adapter find；unknown/unsupported 是 filtering，selected failure 不 fallback，adapter input 不含 project state。
- [ ] 3.4 实现 path + exact nested unit wrapper、每文档至多一个 local failure fact，以及 `(document_position, adapter_page, logical_unit_offset)` private-quantum replay；支持 failure-only、empty-but-continuable、beyond-terminal 和 conservative empty terminal page。
- [ ] 3.5 扩展 protocol/readable output、process mapping 和 invocation-log isolation；project success 显示 local failures 且 exit `0`，project mode 不产生 auto-read 或 routing mechanics log facts。
- [ ] 3.6 更新真实 CLI smoke、Semantic Cases 和 canonical package smoke，保留 explicit-path find 的 argv/request/result/output fixtures。

## Verification

- [ ] 4.1 运行范围匹配的 format、clippy 和 CLI/root/traversal tests，覆盖 project resolution、explicit path/directory、project ignore、nested ignore、ordinary hidden、control dirs、file/directory symlink、large file、directory fanout、UTF-8 byte order、identity collision/unrepresentable 和枚举失败。
- [ ] 4.2 运行 routing/wrapper/replay tests，覆盖 automatic filename/longest-suffix、explicit prefilter、unknown/unsupported filtering、no fallback、same ref across documents、local/fatal split、adapter `limit` 隔离、failure-only/empty continuation、later/beyond-terminal page 和 stable same-build replay。
- [ ] 4.3 运行 request/response schema/example、old-fixture compatibility、raw/readable parity、protocol failure、exit mapping、invocation logging 和 Semantic Case 验证。
- [ ] 4.4 运行真实开发 CLI、`bun run smoke:docnav` 与 canonical package smoke，并确认 explicit-path find 行为完全兼容、project mixed/failure-only facts 可见且 machine stdout 仍单一 response。
- [ ] 4.5 更新 Semantic Case 映射并运行 `bun run test-evidence -- check --root .`；按批准依赖路径运行 license/advisory/MSRV/target/package-size/startup scope checks。
- [ ] 4.6 只有 4.1–4.5 的 schema、实现、测试、真实 CLI、package 和 dependency evidence 全部通过，才按 2.1 登记的 delta 将稳定 owner 同步为 Current；重新运行受影响文档校验和 `bun run verify:docnav-workspace`。
- [ ] 4.7 在 design 追加 `## Implementation Observations`，记录 exact traversal path、private quantum evidence、fanout/replay cost 和未成为 contract 的实现细节，最后审查 scoped diff 与 whitespace。
