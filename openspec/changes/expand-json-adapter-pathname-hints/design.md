本临时 design 规划一项有序的 JSON manifest allowlist 扩展：先以 JSONC predecessor 的 Current 状态重建契约基线，完整保留其两个 descriptor content types，再只改变强 pathname hints 与对应证据，不改变通用 routing 或 JSON operation architecture。

## Context

动机见 [proposal.md](proposal.md)。Current JSON adapter owner 与主 spec 精确声明 `.json`、`.code-workspace` suffixes 以及 `.prettierrc`、`.watchmanconfig` exact filenames；它们只是 automatic selection hints，匹配不读取文档、不证明 JSON validity，也不把 matched hint 传入 adapter strategy。被选中的 `docnav-json` 按其 grammar 和 operation contract 打开实际文档，失败后不重新 route。

Active change `support-jsonc-in-json-adapter` 的 Target descriptor 是一个 `json` identity、`.json` / `.code-workspace` / `.jsonc` suffixes、exact `.prettierrc` / `.watchmanconfig` filenames，以及 `application/json` / `application/jsonc` content types；它还计划让全部 selected documents 使用一套 JSONC-capable grammar，但形成本文时这些 predecessor facts 尚未证明为 Current。`.code-snippets` 常见内容依赖该 grammar，而且两项 change 会触及同一个 registry-facing requirement；因此本文只能保存 combined successor target，不能把当前 main spec 误述为已含 `.jsonc` 或 `application/jsonc`，也不能直接把草案中的 `MODIFIED` block 当作届时的完整基线。

这些候选 pathname 都有稳定的 JSON-family 表示，但部分还带 profile 语义。既有 JSON tree、ref 与 operation surface 足以提供 generic structural navigation；profile validity、domain semantics 和远程资源处理不是 routing hint 或 generic adapter 的责任。

## Goals / Non-Goals

**Goals:**

- 让一个经审计的 manifest allowlist diff 覆盖七个强 suffixes 与两个强 exact filenames，并保持一个 `docnav-json` / `json` identity 以及 predecessor 的 `application/json` / `application/jsonc` descriptor content-type set。
- 让 owner contract、main spec、inspection、selection、Case ledger、targeted tests、coverage 和 release evidence 对相同 observable hint set 与 unchanged predecessor content-type set 闭合。
- 在实现前确认 predecessor 与 then-Current baseline，避免 OpenSpec delta archive 丢失或覆盖相邻 requirement 内容。

**Non-Goals:**

- 不建立 profile registry、per-path grammar mode、schema validator、canonicalizer、domain-specific ref 或 remote resolver。
- 不新增、删除、选择或解释 descriptor/result content type；相关语义完整保留 predecessor 与 JSON adapter owner 的责任。
- 不改写 exact-filename/suffix lookup、normalization、precedence、route-before-document-I/O、explicit selection 或 no-fallback 算法；这些继续引用 Current owner contract。
- 不把未列入 allowlist 的 JSON-like、record-stream、binary 或模糊配置名称作为顺便扩展项。

## Decisions

### Decision 1: Implementation is sequenced after the JSONC predecessor

`support-jsonc-in-json-adapter` MUST 先完成、同步 owner artifacts，并有证据成为 Current；在此之前本 change 的任何 production、owner、test 或 release 实施任务都不得开始。阻塞审计届时 MUST 从 then-Current `openspec/specs/json-adapter/spec.md` 复制完整同名注册 requirement，再把本 change 的 allowlist diff 应用于 `MODIFIED` block，同时完整保留 predecessor 实际采用的 grammar、`application/json` / `application/jsonc` descriptor content-type set、matched-content-type input exclusion 与 evidence 边界。

虽然 `.jsonld`、`.geojson`、`.har`、`.webmanifest`、`.ipynb`、`.sarif`、`Pipfile.lock` 与 `deno.lock` 的 routing-hint 语义不依赖 JSONC，仍选择整体顺序化，而不拆出并行 strict-only change；这样避免两个 active deltas 对同一注册 requirement 产生 archive 顺序依赖，也避免暂时交付一个缺少 `.code-snippets` 的中间 allowlist。

### Decision 2: One closed allowlist distinguishes suffixes from exact filenames

新增 normalized suffixes 精确为 `.code-snippets`、`.jsonld`、`.geojson`、`.har`、`.webmanifest`、`.ipynb`、`.sarif`；新增 case-sensitive exact filenames 精确为 `Pipfile.lock`、`deno.lock`。Then-Current entries（包括 predecessor 实际加入的 `.jsonc`）保持原有顺序；新增 suffixes 与 exact filenames 分别按本句列出顺序追加，不顺便重排 predecessor 数据。

选择强 filename evidence 的 closed set，而不采用 MIME inference、content sniffing、generic basename heuristics 或全量 `+json` profile catalog。`package.json`、`tsconfig*.json`、`deno.json` 等已经由 `.json` suffix 覆盖，不再增加冗余 exact entries。

### Decision 3: Every new match receives only generic structural navigation

Hint 命中只选择既有 `docnav-json`。Outline、ref、read、find、info、full-read、pagination、cost、diagnostic 与 no-fallback 行为继续来自届时 JSON adapter owner contract；pathname 不构成 profile-validity assertion，也不改变 logical tree。JSON-LD context expansion、GeoJSON geometry validation、HAR/Web Manifest/Notebook/SARIF schema semantics、lockfile version semantics 和远程 resolution 均不进入该 adapter contract。

另一方案是为各 profile 增加专用 adapter 或 profile-aware operation，但当前没有对应 user scenario、contract 或 evidence owner；把这些语义藏在 generic JSON adapter 中会扩大 ref/output 兼容面，因此留给独立需求与 change。

### Decision 4: Grammar and content types remain owned by the predecessor and selected adapter

本 change 不定义 strict JSON、JSONC 或任何其它 grammar，也不根据 suffix、exact filename、declared content type 或 parser success 选择 mode。`.code-snippets` 和所有其它新增路径在被选择后都使用 predecessor 落地后的同一 JSON adapter grammar；grammar-invalid input 返回届时 owner-compatible diagnostic，并保持 selected failure no-fallback。

Predecessor Target 已选择一个 `json` descriptor 声明 `application/json` 与 `application/jsonc`，并由 JSON adapter owner 解释 source/result content-type 行为。本 change 完整保留这两个 descriptor values，不新增、删除、推断或重新解释 content type；新增 pathname 与 content type 之间没有 profile、dialect 或 validity mapping。

不会新增第二个 format id、adapter id、capability 或 routing-selected metadata。以 permanent `ADDED` requirement 再拥有完整 hint set 的方案被拒绝；exact set 只在既有注册 requirement 的完整 `MODIFIED` block 中拥有，通用 routing 规则仍由 Current owner 文档拥有。

### Decision 5: The implementation surface is the manifest allowlist, not a routing algorithm

Production diff 应限于 built-in JSON adapter manifest 的 `extensions[]` / `filenames[]` 数据，除非阻塞审计证明 Current 实现已经把该事实集中到另一单一 owner。Core derived indexes、lookup precedence、path normalization 和 adapter dispatch 不因本 change 增加分支、fallback、probe 或依赖。

Process boundary 保持不变：`adapter list` 会投影扩展后的 public manifest facts，包括 unchanged `application/json` / `application/jsonc` descriptor values；automatic selection 内部消费 pathname 并把同一个 closed standard operation input 交给 selected adapter。Protocol/raw/readable envelopes、CLI/env/config inputs、typed fields、invocation log、ref、continuation、content-type semantics 和 schema field shapes 不增加 matched-hint fact，matched content type 也不进入 strategy input。

### Decision 6: Evidence proves the allowlist and representative end-to-end behavior without profile claims

实施时先按项目测试证据流程恢复完整 current tree 与 Case 映射。Table-driven manifest/registry assertions MUST 精确覆盖每个新增 suffix 和 exact filename、一个 `json` identity 以及 unchanged `application/json` / `application/jsonc` descriptor values，并证明 listing 投影、排序/normalization 及 semantic validation 仍闭合；automatic-selection tests MUST 至少用一个新增 suffix 和一个新增 exact filename 执行真实 `outline -> ref -> read`，并以代表性 invalid content 证明 selection/content-type declaration 不等于 validity 且不 fallback。

Owner docs、main `json-adapter` spec、Case ledger、coverage mapping、core CLI smoke 和 release-package inspection/roundtrip 必须同步。高成本 release smoke 可以选择代表性 suffix/exact filename roundtrip，但其 `adapter list` assertion 必须覆盖完整 manifest set；不为每个 domain profile 建立重复 navigation corpus，因为本 change 不承诺 profile semantics。

### Decision 7: Exclusions are part of the closed boundary

JSON5；NDJSON/JSONL；RFC 7464 JSON Text Sequences；含义不明确的 rc names；弱 generic basenames；CBOR、BSON 等 binary JSON-like formats；以及任何 profile-specific navigation 均不加入。本 change 也不把 `.json` 已覆盖的名称或未经确认的候选顺便加入。

若后续需要任一排除项，必须先单独确定其 grammar/framing、document model、ref/continuation、diagnostic、output 和 validation owner，而不是扩大本 allowlist。

## Risks / Trade-offs

- **[Predecessor drift]** JSONC change 归档后注册 requirement、grammar 或 descriptor content-type 边界可能与当前草案不同 → task 0 阻塞审计从 then-Current 完整重建 `MODIFIED` block，逐项保留两个 content types 与 matched-content-type input exclusion，并在审计完成前禁止实现。
- **[Hint false positive]** 某个匹配文件可能损坏、使用未来不兼容版本或不满足 profile → 明确只承诺 adapter grammar 与 generic navigation；返回 owner-compatible parse diagnostic，不 fallback。
- **[False semantic confidence]** 用户可能把成功 outline 误解为 profile validation → owner/spec/readable materials 不得使用“valid JSON-LD/GeoJSON/etc.”措辞，测试也不宣称 domain validity。
- **[Allowlist duplication]** 多处手写 exact set 容易漂移 → manifest 是实现事实源；主 owner/spec 拥有 public contract，inspection、tests、Case 和 release evidence 只同步或断言该事实，不创建第二套 routing registry。
- **[Broader automatic selection]** 原本得到 unsupported-format 的 pathname 可能改为 JSON-owned parse failure → 这是 hint expansion 的预期兼容性变化；回滚时移除新增 hints 并同步证据即可恢复原 routing result。

## Migration Plan

1. 完成 task 0：证明 predecessor Current，恢复 then-Current owner/spec/code/test/release baseline，并重建、严格验证本 change artifacts。
2. 先同步 owner contract、主 spec、Case 与预期 evidence matrix，再建立新增 hints 的 failing assertions。
3. 只更新 manifest allowlist，随后更新 registry/listing、automatic selection 和 release evidence；两个 descriptor content types 只作为 unchanged predecessor facts 被断言。
4. 运行目标 crate/core tests 与 workspace unified verification；依据最终 diff 审核没有 profile、protocol、input 或 routing-algorithm 扩张。
5. 归档前再次把 delta 与 then-Current requirement 对齐。回滚删除这九个新增 hints 及专属证据；无需数据迁移，受影响文件可继续通过 explicit `--adapter docnav-json` 使用 generic navigation。

## Open Questions

无未回答开放问题，可以进入实现前审计；该审计本身仍是阻塞门禁，不表示本 change 已批准实施或已成为 Current。
