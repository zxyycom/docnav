本临时proposal计划让`docnav-json`在接受闭合JSONC语法的同时，把direct comments与independent tail comments都通过`outline -> ref -> read`交付；语法接入是这一完整结果的基础，而不是独立产品阶段。Tail public package与root-tail grouping已经闭合，exact parser/model仍由blocking implementation audit选择。

## Why

编辑器、workspace 和工具配置经常使用 comments 与 trailing commas，但 Current `docnav-json` 只接受 strict JSON。若只放宽 parser 而仍从正常导航中丢弃注释，读取者会失去配置键的意图、约束和用法；若先独立选择 syntax-only parser，又可能在后续 attribution 时增加第二套 scanner 或重建 source model。因此 parser、source evidence、comment attribution 和导航表示需要由同一个 change 的端到端契约驱动。

## What Changes

- 让同一个 `docnav-json` strategy 接受 strict JSON 加闭合 JSONC 扩展：`//`、`/* ... */` 与非空 object/array 的单个 trailing comma。Strict JSON 继续是该 grammar 的子集；不增加 pathname-selected dialect、parse retry、第二个 adapter 或第二个 format identity。
- 在现有 `json` descriptor 中增加 `.jsonc` suffix 和 `application/jsonc` source content type；`.json`、`.code-workspace`、exact filename 与 explicit adapter selection 均使用同一个 grammar。Pathname hint 仍只负责选择，不证明内容有效。
- 让 JSON adapter 的单一 source-aware model 同时保留 logical JSON tree、member/element/value regions、raw number tokens 与 ordered comment spans，并按 JSON-owned placement rules 将 comments 唯一归属于一个 navigation binding或tail anchor。Empty-container comments归 container value自身的direct binding；每个 canonical tail anchor至多拥有一个tail-comment bundle。Ref resolution在该 primary model上形成selected-first borrowed selection chain，为每一级frame保留binding/value/direct-comment/tail-comment context；parser/CST attachment只提供证据，不自动成为产品归属规则或renderer policy。
- 对有 direct comments 的 logical entry 返回 bounded comment `summary`，并生成 canonical direct-comment ref；无direct comments的logical entry继续生成Current base ref。Root container只有在拥有root direct comments时才新增 `<root>` direct-comment entry。每个非空tail-comment bundle另生成tail anchor的末位virtual entry与canonical tail ref。三种view共享canonical logical path，但选择不同projection。
- 让 direct-comment 与 tail ref 的 `read` 返回完整、确定性的 `application/jsonc` document：按 source order 放置所选 direct或tail comment tokens，再放置logical value或tail anchor value的规范化strict-JSON serialization。Base ref 的 read 继续返回 `application/json`，且不包含 comments 或 trailing commas。
- 让完全位于 direct-comment span 内的 `find` occurrence 返回该 binding 的 direct-comment ref，完全位于 tail-comment span内的 occurrence返回tail anchor的tail ref；其它 occurrence继续使用 Current source-region ownership。Source full-read 保留 BOM-stripped 原文，info/full-read 仅在 source 实际含 JSONC-only syntax 时报告 `application/jsonc`。
- 保持 shared protocol envelope、`Entry`、`ReadResult`、generic `readable-view`、pagination 和 cost shape 不变；本 change 复用 `Entry.summary`、opaque ref 与既有 content block，不增加 shared field、operation、output mode 或 caller option。
- 由完整 comment-aware contract 反向选择一个最小 sufficient parser/model 实现，并证明 strict/no-comment JSON、source offsets、duplicate/depth/raw-number behavior、bounded work 与 broader JSON syntax rejection 不回归。

## Non-Goals

- 不把 tail-comment entry扩展成 logical JSON node、JSON Pointer token、可写 comment identity或按源码 offset持久化的 ref；它只是由 canonical tail-anchor path与tail view共同标识的virtual navigation selection。
- 不支持 JSON5、single quotes、unquoted names、missing commas、multiple roots、JSON Lines/NDJSON、profile validation/canonicalization、remote resolution 或 binary JSON family。
- 不编辑、格式化或写回 comments，不提供 comment mutation API，也不从 comments 推断 schema、type、default 或 validation rules。
- 不增加 `expand-json-adapter-pathname-hints` 拥有的其它 JSON-family suffix/filename hints，也不预选 `add-json-readable-renderer`、`redesign-find-result-model` 或 `reuse-adapter-document-state` 的未落地语义。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `json-adapter`: 扩展 selected-document grammar、private source model、JSON ref/view grammar、outline/read/find、info/full-read、diagnostics 与验证契约，使 direct comments 与 tail comments 都成为可继续读取的 JSONC navigation facts。

## Impact

- 预期影响 `crates/adapters/json` 的 manifest、parser/source model、traversal、ref、read、find、info/full-read 和测试；parser 选择可能修改 workspace dependency 与 lockfile。
- 实施时需要同步 JSON adapter owner 文档、主 spec、语义 Case、adapter/core/output tests、protocol examples 中使用既有字段的新 JSONC facts，以及 release-package smoke；在代码证据完成前这些都保持 Target 状态。
- Shared ref owner 继续把 ref 当 opaque string 原样传递；shared protocol 和 output shape 不变。若实现审计证明必须增加 shared field 或 operation，本 proposal 必须先扩展 capability scope 并取得独立批准，不能在实现中顺带修改。
- 本 change 取代此前按 syntax support 与 comment surfacing 拆分的计划；其它 active change 只能依赖本 change 成为 Current 后的完整 JSONC grammar/navigation baseline。
