**本 proposal 只为省略 path 的显式 `docnav find` 增加有限、可继续的 current-project 搜索；它保持显式 path 的单文档契约，并以两个活动 change 的最终 owner contract 作为实施前置。**

## Why

当前 `docnav find <path> --query <text>` 一次只能搜索一个文档。调用方若要搜索项目，必须自行发现文件、复制 adapter selection、合并不同格式的结果、组合 document path 与 opaque ref、处理部分失败并发明分页边界；这些工作既重复 core/navigation 责任，也容易破坏 `outline -> ref -> read` 的身份契约。

## What Changes

- 让 `docnav find` 的 path 可选：显式 path 继续走原有 single-document find；省略 path 时只搜索现有规则解析出的 current project root，显式目录不成为 project alias。
- 新增 core/navigation-owned project discovery 与 orchestration：按 per-directory sorted deterministic DFS 惰性遍历 project-local eligible files，不跟随 symlink、不按文件大小跳过，并且不预收集完整项目。
- automatic project discovery 对每个候选只消费 `replace-probe-traversal-with-inferred-routing` 最终批准的一次 inference → normalized format → exact adapter seam；explicit adapter intent 先做 exact id lookup，再用该 adapter manifest format descriptors 做 discovery prefilter，真正支持性仍由 selected adapter 的实际 parse/find 决定。
- 新增 raw `find` request closed union：原有 `document.path` request encoding 原样保留为 single-document branch；project branch 改带 resolved `project.root` 且禁止 synthetic `document.path`。两条 branch 共享现有 find arguments，但 project envelope 不进入 adapter。
- 新增 project find success branch：每个 result unit 以独立 normalized `document.path` 包住 `redesign-find-result-model` 最终单文档 logical unit，并保持 nested adapter ref opaque；document-local failures 是 bounded success facts，project/global failures继续使用顶层 failure envelope。
- adapter-owned `limit` 只继续约束每次 single-document dispatch。Project owner 使用一个正数、有限、在同一 build 内固定但 exact value 保持 implementation-private 的 work quantum，用 numeric page deterministic replay 在 `(document position, adapter page, logical-unit offset)` 上推进；允许 empty-but-continuable adapter page 形成 empty project result + continuation，不新增 cursor、cache 或 snapshot。
- Project mode 禁用 auto-read；readable/protocol 输出从同一个 immutable project response 派生。
- 依赖引入不是 proposal 的既定结论。任何 walker/ignore dependency 只可作为候选，必须先完成 ecosystem、maintenance、security、license、MSRV、targets、package/startup 和 alternatives 调查，再由人工批准精确 dependency/version/features 或 no-new-dependency 路径；未批准不得修改 Cargo manifests 或 lockfile。
- 实施必须等待 `redesign-find-result-model` 与 `replace-probe-traversal-with-inferred-routing` 的 owner acceptance、implementation 和 validation；两项 acceptance 都是本 change 的可勾选 blocking tasks。

### Non-goals

- 不修改 occurrence、distinct exact-ref/node、group、evidence、multiplicity、单文档 ordering、单文档 continuation 或单文档 auto-read 模型。
- 不新增持久索引、跨运行 cache、后台 daemon、relevance ranking、query language、fuzzy search、`fast-find`、实时 progress 或省略 `find` subcommand 的 query routing。
- 不按文件大小静默排除受支持文档，不为慢文件/adapter 增加任意 wall-clock timeout，也不并行 dispatch。
- 不跟随 symlink、搜索 project root 外部、读取 user-global ignore state，或新增 Docnav ignore config language。
- 不新增 project-aware adapter operation，不把 project root/page/state 序列化进 adapter options，不解析或改写 adapter ref。
- 不在本 change 中选择 inference library、walker library或修改两个 predecessor change。

## Capabilities

### New Capabilities

- `project-find`: 拥有项目范围 discovery、routing orchestration、path + opaque ref identity、per-directory deterministic order、fixed-quantum numeric continuation、局部失败和 project-mode non-goals。

### Modified Capabilities

- `core-cli`: 使 `find` path 可选，省略时选择 current project root，并保持显式 path、strict directory failure、help/argv 和 process mapping。
- `navigation-input-resolution`: 增加 project scope handoff、predecessor-owned inference/manifest routing seam、逐文档 selected view/dispatch、fixed-quantum replay和 auto-read exclusion。
- `protocol-contract`: 增加 backward-compatible single-document/project find request closed union，以及 project success、document-scoped match/failure 和 numeric page shape。
- `output-contract`: 为同一 project find response 定义 protocol-json passthrough 和 readable-view projection。

## Impact

- Affected public surfaces: `docnav find` command arity/help、raw project find request、project-mode protocol result、readable output、numeric continuation、partial-failure presentation and exit behavior.
- Affected owner/implementation areas after gates pass: core project context/path handling、`docnav-navigation` discovery/routing/orchestration、shared protocol/output types、request/response schemas and examples、CLI smoke、Rust owner tests、Semantic Cases and release validation.
- Adapter contract、format-specific query/ref grammar and single-document find implementation remain unchanged by this change; project orchestration invokes the selected adapter through its existing single-document strategy.
- `redesign-find-result-model` is a blocking semantic predecessor. Its owner must accept the exact nested logical-unit handoff and finish implementation/validation before this change applies.
- `replace-probe-traversal-with-inferred-routing` is a blocking routing predecessor. Its owner must accept the exact normalized format → registry seam and manifest-descriptor handoff and finish implementation/validation before this change applies.
- `audit-runtime-performance-boundaries` remains independent: it may attribute traversal, inference, parse/search, replay or output cost, but it does not select dependencies or turn observations into an implicit timing gate for this change.
