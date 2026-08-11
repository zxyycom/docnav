# Design

设计由 core 选择 project scope，navigation 惰性发现并逐文档路由，project layer 只包装稳定单文档 find unit 并用 numeric replay 继续。

## Context

- Current CLI find 要求显式文档 path，adapter contract 只接受单文档 operation。
- Current automatic routing 在 target-document I/O 前按 pathname hints 选择 exact adapter；selected parse/find failure 不 fallback。
- Current find result 仍是 occurrence-oriented `Entry`，但 `redesign-find-result-model` 将决定最终逻辑单元。本计划不能固化其 provisional 字段。
- Project root 已由 core project context 解析；project mode 使用该 root，不构造 synthetic document path。

## Goals / Non-Goals

Goals:

- 提供有限、确定性、可继续的 current-project find。
- 保持每个 result 的 normalized document identity 与 adapter-owned opaque ref。
- 在不建立跨运行状态的前提下重放遍历与单文档分页。
- 让局部失败可见而不丢弃其它文档结果。

Non-Goals:

- 不改变单文档 find 模型、adapter query/ref grammar 或 adapter `limit`。
- 不建立索引、cache、daemon、并行 dispatch、wall-clock timeout、fuzzy/ranking 或 ignore config language。
- 不跟随 symlink、访问 project root 外部、读取 user-global ignore state 或静默跳过大文件。

## Decisions

### 1. Path presence 是唯一 CLI scope discriminator

`docnav find <path> --query <text>` 沿用 single-document branch；`docnav find --query <text>` 使用本 invocation 已解析的 `ProjectContext.project_root`。显式目录继续按现有 document-path failure 处理；query 内容、cwd 文件数、像路径的 query token 和 adapter outcome 都不能反向选择 scope。

### 2. Raw find request 使用 closed union

Public raw request 是两个互斥 closed branch：

```text
SingleDocumentFindRequest {
  protocol_version, request_id, operation: "find",
  document: { path }, arguments: FindArguments
}
| ProjectFindRequest {
  protocol_version, request_id, operation: "find",
  project: { root }, arguments: FindArguments
}
```

Single-document branch 的字段名、required fields 和 `FindArguments` encoding 保持兼容。Project branch 的 `project.root` 是已解析的 normalized project-root string，且没有 `document`。Both/neither target 必须失败；不增加 `scope` input、synthetic document path 或 repair heuristic。Project envelope 只到 core/navigation，adapter 仍接收现有 closed single-document `FindInput`。

### 3. Core 拥有 scope，navigation 拥有 project orchestration

Core 拥有 argv/help、scope discriminator、project-root resolution、output plan 和 process mapping。Navigation 拥有 project-local lazy traversal、deterministic position、逐 candidate routing、selected config projection、single-document dispatch、result wrapping、local/fatal classification、fixed-quantum replay 和 page projection。

每次 adapter 调用只接收一个 normalized document path、query、adapter-owned `limit`、adapter page 和 applicable typed options；不得接收 project root、candidate list、outer page、traversal position、cross-document accumulator、auto-read mode 或 output strategy。

### 4. Traversal 是 per-directory sorted deterministic DFS

Traversal 递归考虑 project root 下的 regular files，应用 project `.gitignore`、nested `.gitignore` 和 `.ignore`，忽略 user-global Git ignore/exclude state；始终排除 `.docnav`、`.git`、`.hg` 和 `.svn` control directories，ordinary hidden entries 仅在 project-owned rule 命中时排除。Traversal 使用 symlink metadata 并跳过 file/directory symlink，不按大小静默排除 regular file。

每个目录只读取并 buffer immediate entries，派生 lossless normalized path segments，按 case-sensitive UTF-8 bytes 排序后 depth-first 访问。DFS 可以保留 ancestor sibling state，但不得构造或排序 flat all-project candidate list。目录枚举、owned ignore source 或 identity normalization 不能支持确定性 replay 时是 fatal，不得静默漏项。

### 5. Routing 复用 Current exact-selection seam

Traversal 前先校验 global registry/catalog invariants；duplicate/conflicting normalized identities 是 global fatal，registry order 不选择 winner。

Automatic branch 对每个 candidate 复用 Current complete-basename routing：先 exact filename，再 ASCII-normalized longest end-anchored suffix，由 matched hint 映射 normalized format identity 后 exact lookup 唯一 definition。无 pathname hint 或没有 exact registered adapter 是普通 filtering；routing 不读取 document metadata/content。选择成功后的 path/access、adapter acquisition、parse、semantic、find 或 result-validation failure 是一个 bounded local failure，且不得 fallback。

Explicit adapter intent 在 traversal 前 exact id lookup 一次并跳过 automatic routing；missing id 保持现有 diagnostic。选定 manifest 的 format hints 只作为确定性 path-eligibility prefilter，eligible file 仍必须由选定 adapter 真实处理；descriptor 不是内容有效证明，selected failure 不 fallback。

### 6. Project unit 只包装单文档 unit

Project success 使用：

```text
scope: "project"
matches[]:
  document: { path }
  match: SingleDocumentFindUnit
failures[]:
  document: { path }
  error: ProtocolError
page: positive integer | null
```

`SingleDocumentFindUnit` 必须来自 `redesign-find-result-model` 最终实现后的 Current contract；本 change 不选择 occurrence/node/group/evidence/multiplicity 字段。`document.path` 使用现有 normalized slash-path contract，nested ref 保持完整 opaque。相同 ref 在不同文档由 `(document.path, match.ref)` 区分，并可传给普通 explicit-path read；shared layer 不拼接、解析或跨文档去重 ref。无 match 的文档不产生 wrapper，每个失败文档至多产生一个 failure wrapper。

### 7. Failure 分为 local success fact 与 global fatal

唯一 document identity 形成后的 candidate metadata/open、selected adapter acquisition/parse/semantic/find 和 selected result validation failure，形成至多一个 existing diagnostic projection 并推进该文档。Unknown/unsupported pathname、explicit descriptor mismatch 和 valid no-match 是 filtering/outcome，不是 failure。

Invalid argv/request/query/config/catalog、unresolved/unreadable project root、root/nested directory 或 owned ignore-source enumeration failure、unrepresentable/colliding identity、explicit adapter lookup failure、registry invariant、project result validation 和 output preparation 保持 top-level fatal。Validated mixed、failure-only 或 empty-continuable project result 都是 success/exit `0`；local failures 必须同时进入 protocol/readable facts，不能只写 stderr 或拆成 sibling envelope。

### 8. Adapter limit 与 project work quantum 分离

Resolved positive `limit` 继续作为每次 single-document adapter dispatch 的 adapter-owned result budget；project orchestration 不把它解释成 discovery、dispatch 或 outer-result quota。

Project owner 使用一个正数、有限、不可配置、同一 build 固定且 implementation-private 的 quantum。每个 transition 最多输出一个完整 match/failure wrapper，并必须推进 `(document_position, adapter_page, logical_unit_offset)` 至少一项：过滤或 terminal page 推进 document；local failure 输出一次后推进 document；unit 输出推进 offset；continuable adapter page 推进到 exact returned page 并重置 offset；empty-but-continuable page 也必须推进。Invalid adapter pagination/result 降为该文档 local failure 而不能循环。

每个 project page 的 transition 数与 match/failure wrapper 总数均不超过 quantum；exact value 不进入 CLI/config/protocol/schema/example 或跨 build compatibility。Directory immediate-entry buffer、一次 routing/adapter call 的 time/bytes 不由 quantum 静默截断。

### 9. Numeric replay 重建状态

Project request page 默认为正整数 `1`。回答 page `n` 时从 `(0, 1, 0)` 开始，以同一 build quantum 重放前 `n - 1` 个 logical page step 并丢弃 earlier outputs，再执行第 `n` 步。Quantum 用尽而 terminal 未证明时返回 request page + 1；terminal 已证明时返回 `null`；请求超过 terminal 返回 empty matches/failures 与 `page: null`。

Filtering-heavy 或 empty-but-continuable adapter page 可以产生 empty project page + non-null continuation。稳定 root/query/adapter intent/options/limit/tree/content/project ignore state 在同一 build 重放相同边界；跨 invocation mutation 按新 Current state 处理，不承诺 snapshot。允许保守的额外 empty terminal page；不增加 opaque cursor、result-set id、persisted traversal state 或 cross-run cache。

### 10. Project mode 不 auto-read

Project result 不包含 `auto_read`，不计算 query-global/composite uniqueness，也不 dispatch nested read。Project selected view 不 materialize `defaults.auto_read`；显式 project `--auto-read` 在 discovery 前以 scope-inapplicable input 失败。Full config validation 仍可识别合法 configured value，但它不改变 project orchestration。调用方使用 document path 与 opaque ref 显式 read。

### 11. Raw/readable 从同一 project response 派生

Outer operation 保持 `find`；single-document success branch 保持兼容，project request 返回 required `scope: "project"` branch。`ProtocolJson` 序列化 immutable response；built-in renderer 显示 scope/page、独立 document path、完整 opaque ref、nested unit facts 以及 local failure path/code/message。Renderer 不 dispatch adapter、不拼接 path/ref、不隐藏 failure、不生成 display-only identity 或 auto-read content。Closed schema/types 的 exhaustive consumers、request/response examples 和 validation tests 必须同步更新，旧 single-document fixtures 继续原样通过。

### 12. Predecessor handoff 单向

Current pathname routing 已是稳定输入，不是待实施 predecessor。本计划唯一产品硬前置是 `redesign-find-result-model`：该 Change 完成最终 contract、实现和验证后，本计划从届时 Current owner/types/schema 取得 exact nested unit、ordering、continuation 和 auto-read seam，并重写全部 overlapping assumptions。

本计划不修改 predecessor artifacts，也不把 project traversal/result ownership 反向放入 predecessor。Runtime performance audit 可以消费测量但不是实施前置；后续 Current routing 或 document-lifecycle 变化只触发 scoped re-audit。

## Risks / Trade-offs

- Numeric replay 会重复 traversal，later-page cost 随页码增长；以有限 quantum、稳定 order 和无跨运行状态换取简单、可恢复 continuation。
- Sorted DFS 内存受最大单目录 fanout 和 active ancestor sibling state 影响，但不应与全项目文件数形成 flat collection；dependency audit 必须测量。
- Quantum 不约束一次目录枚举、routing 或 adapter parse 的 wall-clock/bytes；保持完整可观察行为，不用 silent large-file skip 伪装性能保证。
- Ignore/walker dependency 会影响 supply chain、license、package size、startup、MSRV、targets 和行为；必须经过证据与人工批准，也允许 no-new-dependency。
- Local failure + exit `0` 可能被忽视；两种 output、examples 和 smoke 必须显式显示 mixed/failure-only facts，只有 identity 后的文档局部失败可降级。
- Project mutation 会改变 numeric replay page boundary；契约只承诺 stable-state same-build replay，不承诺 snapshot。
- Closed request/result union 要求 exhaustive consumers 原子更新；旧 single-document fixtures 是兼容门禁。
- 单文档模型变化会使 wrapper 失配；该 handoff 是明确 gate，完成后从 Current owner/types/schema 重写 nested unit。

## Open Questions

以下门禁均有 Implementation 1.x 的 owner、关闭动作和被阻塞任务：

1. `redesign-find-result-model` 的最终 logical unit、ordering、continuation 和 auto-read seam 是否已经实现并验收？由该 Change 的 Current owner/实现/schema evidence 关闭；未关闭时阻塞 2.1 及之后任务。
2. 获批 traversal implementation 是哪个 exact crate/version/features，还是 no-new-dependency？由用户或指定 architecture owner 在 dependency audit 后关闭；未关闭时禁止 Cargo/lockfile 和 production traversal 修改。
3. 哪个正数、有限、同一 build 固定的 private quantum 通过 empty/filter-heavy/multi-page/local-failure replay/progress workload？由本 Change 的验证证据关闭；未关闭时阻塞 production pagination。
