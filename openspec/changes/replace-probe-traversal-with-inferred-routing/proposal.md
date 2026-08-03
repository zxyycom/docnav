本 proposal 定义 `replace-probe-traversal-with-inferred-routing` 已批准但尚未应用的 Target：用 manifest-owned pathname hints 和精确 registry lookup 替代 adapter probe 遍历，并以零新增 routing dependency、显式强制选择和完整 probe 删除为实施边界。实施前审计已经完成；本文不表示 Current 已迁移。

## Why

Current automatic selection 按 static registry 顺序逐个执行 adapter probe，并把前序候选失败当作可恢复证据；因此选择结果依赖注册顺序，每增加一个 adapter 都会扩大候选执行面，也让 selection 与真实 adapter 执行形成两套分散入口。现有 Markdown 与 JSON manifests 已拥有 format identity 和 extensions；常见 project config 还需要 exact filename hints。Automatic selection 应收敛为一次无 I/O 的 pathname lookup，同时保持 selected adapter 对真实 parse 和 operation 语义的所有权。

## Status and Authority

- **已确认**：automatic routing 只使用 manifest-owned basename suffixes 与 exact filenames；在 route 命中前不对目标文档执行 metadata、open、canonicalize、read 或 parse，不新增 routing dependency，也不保留 probe、compatibility、inspection 或 adapter fallback path。
- **已确认**：exact filename 做大小写敏感的完整 basename match，并优先于 suffix；`extensions[]` 作为可含多个点的完整-basename suffix，按 ASCII 大小写归一化比较，多个 suffix 同时命中时最长者优先。归一化后完全相同的 suffix、同类 filename 重复声明和 duplicate format identity 是 release blockers，runtime 只保留 global internal defense。
- **已确认**：无 pathname hint 时只返回 `FORMAT_UNKNOWN + FORMAT_NOT_RECOGNIZED`；manifest-derived index 不产生“识别出 format 但 registry 无 adapter”或 document-level ambiguity。Explicit adapter intent 跳过 automatic routing并强制 selected adapter 执行真实解析。
- **已确认**：probe 删除后，selected JSON syntax、trailing-input、duplicate-member 与 depth failure 使用新增 `DOCUMENT_CONTENT_INVALID` 和 JSON-owned stable reason；routing 不把它们重写成 selection 或 internal failure。
- **证据 owner**：正式调查主题 `docs/investigations/dependencies/format-routing-inference.md` 拥有候选功能、重量、生态、活跃度、pathname alias 复查和限制证据；它不取代本 proposal/design 中的批准状态。
- **决策 owner**：活动决策 `docs/decisions/adapter-evolution/route-by-manifest-basename-hints.md` 保存跨 change 的批准方向，并在本 change 应用前保持 `unaligned`；本 proposal/design 保存本 change 的 exact Target。
- **实施就绪状态**：tasks 0.1–0.11 已完成，sections 1–7 可以按 tasks 的测试、同步、实现和验证顺序开始；尚未把 Target 应用到 Cargo、lockfile、production code、owner docs、schema、examples、fixtures、tests 或 release artifacts。

## What Changes

- **BREAKING**：automatic selection 从“先访问/规范化文档，再按 registry 顺序 probe 并选择第一个 `supported: true`”改为“调用 pathname → lexical routing basename → manifest-derived exact-filename/normalized-suffix lookup → normalized format identity → exact registry lookup → filesystem-backed path/access normalization → dispatch”。No-match 在任何目标文档 filesystem I/O 前结束；registry 顺序和 document bytes 不再影响 selection。
- Manifest format descriptor 新增 `filenames[]`，用于 `.prettierrc`、`.watchmanconfig` 等 exact basename hints；`extensions[]` 继续声明 `.json`、`.md`、`.markdown`、`.code-workspace` 等带前导点的 basename suffix，并允许 compound suffix。Exact filename 优先；suffix 对完整 basename 做 ASCII 大小写归一化后缀比较，最长命中优先。因此 `model.schema.JSON` 可让 `.schema.json` 覆盖 `.json`，而 `settings.json.backup` 不命中 `.json`。
- 没有 pathname hint 时使用现有 `FORMAT_UNKNOWN` + exact `FORMAT_NOT_RECOGNIZED` details。由于 hint index 与 format index都从同一 static registry manifests 派生，automatic routing 不产生 `NO_SUPPORTED_ADAPTER + format` 或 `FORMAT_AMBIGUOUS`；重复 hint/format 声明在 construction、doctor 与 release validation 阶段阻断，逃到 runtime 时使用 global `INTERNAL_ERROR`。
- 显式 `--adapter` 继续表达 caller intent：按 adapter id 精确 lookup，跳过 automatic pathname routing；lookup 成功只确定 strategy，selected adapter 仍必须在真实 operation 中读取并 parse 文档。
- selected adapter 的 parse、semantic validation 或 operation failure 是该次执行的最终 adapter diagnostic；automatic 和 explicit path 都不得回退到其它 adapter。JSON syntax、trailing input、duplicate member 与 maximum-depth failure 迁移为 `DOCUMENT_CONTENT_INVALID`，分别使用 `JSON_SYNTAX_INVALID`、`JSON_TRAILING_INPUT`、`JSON_DUPLICATE_MEMBER` 与 `JSON_MAXIMUM_DEPTH_EXCEEDED`；non-UTF-8 继续使用 `DOCUMENT_ENCODING_UNSUPPORTED`。
- 不引入外部 format/MIME/regex dependency、custom inference trait、confidence scoring framework、adapter callback 或第二套路由 metadata。Implementation 只建立从 manifests 确定性派生的私有 exact-filename、normalized-suffix 与 format lookup view；matched hint/format 不进入 adapter operation input。
- Pathname hint 不是内容真实性声明：`.md` 中写 JSON 仍选择 Markdown，`.json` 中写 Markdown 仍选择 JSON；known hint 的 missing、empty、malformed、non-UTF-8 或 JSONC/YAML 内容由 selected adapter 返回正常 diagnostic。JSONC grammar support 由独立 `support-jsonc-in-json-adapter` change 规划，本 change 只建立 routing handoff。
- **BREAKING**：本 change 明确不提供 probe backward-compatibility path，完整删除 `Adapter::probe`、`AdapterDefinition::probe`、`ProbeResult`、probe decode/runtime validation、typed-field projection、schema/examples/fixtures 和 probe-only candidate evidence。实施前仍必须完成 removal inventory，以确保仓库内旧 consumer、文档、验证和 release material 全部迁移或删除；发现额外 consumer 不改变完整删除目标，也不授权保留 inspection surface。
- 先读取并消费正式调查主题 `docs/investigations/dependencies/format-routing-inference.md`，再完成本 change 的 probe removal inventory。正式调查报告覆盖功能充分性、依赖重量、生态采用度/活跃度、pathname alias coverage、maintenance、security、license、MSRV、targets、binary/package size、startup 和 alternatives；本 change 不拥有或复制报告。

## Non-Goals

- 不公开 inference enum、library error/text、confidence 或 detection trace；public contract 只观察 project-owned selection result/diagnostic。
- 不建立可插拔 detector framework、自定义评分/阈值系统、adapter-owned inference hook、fallback chain 或 content-type registry。
- 不改变 adapter-owned ref、parse、outline/read/find/info、full-read、pagination、success envelope 或 readable output 语义；selection failure 的既有 canonical diagnostic details 会按 owner delta 精确更新。
- 不在本 change 中让 JSON adapter 接受 comments、trailing commas、JSON5、NDJSON/JSON Lines 或其它 JSON-family grammar；这些能力需要各自的 parser/contract change。
- 不在本 change 中定义 adapter 内部 parser mapping、dialect selection 或共享私有事实源；对应 adapter/change 在不依赖 routing match 的前提下拥有这些实现选择。
- 不实现 project-wide find；本 change 是 `add-project-wide-find` 的实施前置，只提供其 per-document routing/failure classification，不接管 project discovery/result/pagination。
- 不在本 change 中实现 document-state reuse、runtime performance audit、code adapter 或 JSON adapter 的 owner work。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `adapter-contract`: 固定 adapter surface 完整删除 probe；format descriptors 新增 exact filename hints，core registry validation 阻断 duplicate format identity/path hint，真实 parse 仍由 selected strategy 拥有。
- `core-cli`: 把 pathname routing 移到目标文档 metadata/open/canonicalize/read 之前；命中 adapter 后才执行既有 filesystem-backed document path/access normalization。
- `docnav-architecture`: navigation/core 从 manifests 派生 pathname/format index 并拥有 selection；adapter 只拥有 selected 后的真实 parse、format semantics、ref 与 operations。
- `navigation-input-resolution`: automatic selection 改为单次 lexical pathname lookup 和精确 format lookup，explicit selection 跳过 automatic routing，且 selected adapter failure 不再触发候选 fallback。
- `diagnostics-contract`: automatic no-match 复用 `FORMAT_UNKNOWN`，registry conflicts 使用 `INTERNAL_ERROR`，selected invalid JSON 使用 `DOCUMENT_CONTENT_INVALID`；删除 probe-only candidate vocabulary，并使 `NO_SUPPORTED_ADAPTER`/`FORMAT_AMBIGUOUS` 不再成为本 routing path 的 Target outcomes。
- `protocol-contract`: 以既有 failure envelope 投影 routing outcomes 与 selected-content failure，并从 shared protocol surface 删除 probe result。
- `contract-validation`: 验证 manifest filename hints 和 routing uniqueness，并删除 probe JSON schema/runtime validator及其 validation materials。
- `typed-fields`: 删除 probe consumer-local field definitions/projections，并明确 private pathname/index outcome 不进入 typed-field catalog。
- `markdown-adapter`: 删除 Markdown-owned selection probe 义务；Markdown 只在被精确选中后执行既有 document operations。
- `json-adapter`: 删除 JSON-owned selection probe 与 post-probe reload 特例；manifest 声明获批 JSON pathname hints，selected JSON operation 继续执行当前严格 JSON parse 与 JSON-owned validation。

## Impact

- 计划中的 implementation surfaces：core CLI 的两阶段 pathname/path-I/O sequencing、`crates/shared/navigation` routing、`crates/shared/adapter-contracts` 固定 strategy/definition、core static registry validation/lookup、shared protocol/contract-validation/typed-field consumer、内置 adapters、diagnostic/schema/examples/fixtures 与相关 tests；blocking removal inventory 枚举完整迁移面后整体删除 probe surface。
- Dependency surface：routing 新增依赖为零；Cargo manifests、lockfile 和 license materials不因 pathname routing 改变。
- Diagnostics：navigation 产出单一 primary selection diagnostic；selected JSON content failure 产出 owner-owned `DOCUMENT_CONTENT_INVALID`；exact code/details 由 `diagnostics-contract`、`protocol-contract` 与 `json-adapter` delta 共同固定，且不保留 registry-order candidate evidence。
- Cross-change handoffs：`add-project-wide-find` 的 Decisions 5/12 已把本 change 记录为 predecessor并接受 filter/local/fatal planning seam；routing task 0.6 只核对并记录该 artifact-level acceptance，不等待 project task 1.3、implementation 或 validation。`reuse-adapter-document-state` 与 `add-ast-grep-code-adapter` 是 downstream rebase consumers：本 change 记录 no-probe handoff，不要求它们先选择内部机制、依赖或完成实现；它们推进时必须按最终 Current routing contract 重写旧 probe/candidate-traversal 基线。`support-jsonc-in-json-adapter` 是 downstream grammar consumer：它可以扩展 JSON pathname hints 和 parser semantics，但不得把 content detection 或 fallback重新放回 routing。`audit-runtime-performance-boundaries` 只接收 measurement handoff；archived `add-json-adapter` record 只提供 probe/TOCTOU migration input。
