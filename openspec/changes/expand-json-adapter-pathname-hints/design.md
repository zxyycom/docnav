本 design 记录一个 manifest-only JSON pathname-hint 扩展的决策与理由：在实施前的 JSONC-capable `docnav-json` descriptor 上追加九个高置信度 hints，并用 owner、tests、CLI 与 release evidence 证明它们只扩大 automatic selection，不改变 parser、navigation 或公共契约。实施状态由 [README.md](README.md) 和 [tasks.md](tasks.md) 记录；Current 产品契约仍由长期 JSON owner 与 main spec 定义。

## Context and Pre-change Current Baseline

实施开始时，implementation、tests 与主 `json-adapter` spec 已证明以下 Current 基线：

- adapter id 为 `docnav-json`，唯一 format id 为 `json`；
- `extensions[]` 为 `.json`、`.code-workspace`、`.jsonc`；
- `filenames[]` 为 `.prettierrc`、`.watchmanconfig`；
- `content_types[]` 为 `application/json`、`application/jsonc`；
- 所有 pathname hint 与 explicit adapter selection 使用同一个 JSONC-capable grammar；
- pathname hint 只选择 adapter，不读取或验证文档；selected failure 不重新 routing 或 fallback；
- matched pathname、content type 和 format identity 不进入 adapter strategy input。

Current 实现状态始终由 code、tests 和 release evidence 证明。实施时，task 1.1 先把长期 JSON owner 与上述 pre-change baseline 对齐，再建立本 change 的 Target contract 与失败证据。

## Goals and Non-Goals

**Goals**

1. 用一个有序、closed allowlist 增加七个 suffixes 与两个 exact filenames。
2. 让 manifest、owner contract、main spec、Cases、tests、CLI 与 release evidence 对同一可观察集合闭合。
3. 保持一个 adapter、一个 format identity、一个 grammar 和现有 generic navigation。

**Non-Goals**

- 不增加 profile registry、per-path grammar mode、schema validator、canonicalizer、domain-specific ref 或 remote resolver。
- 不新增、删除、选择或解释 descriptor/result content type。
- 不修改 pathname normalization、lookup precedence、routing、explicit selection 或 no-fallback algorithm。
- 不扩展到 closed allowlist 之外的 JSON-like、record-stream、binary 或模糊配置名称。

## Decisions

### Decision 1: Rebase from the Current JSONC requirement before implementation

本 change 的 `MODIFIED` requirement 完整继承 Current 主 `json-adapter` requirement 的正文和 scenarios，再只应用 pathname allowlist delta。Current `.jsonc`、`application/jsonc`、统一 grammar、matched-input exclusion 与 public-input boundary 都是既有基线，不是本 change 新增的能力。

Task 0 已对照 code、tests、main spec 和 CLI/release evidence 重建并验证该 delta。同步过程中必须区分长期 contract owner 与 Current 实现证据，不得把本 change artifact 或任务勾选本身当作实现证明。

### Decision 2: One closed allowlist distinguishes suffixes from exact filenames

本 decision 采用以下精确有序集合；验收后，同一集合成为长期 owner 中的 Current contract：

| 字段 | 决策采用的有序值 |
| --- | --- |
| `extensions[]` | `.json`、`.code-workspace`、`.jsonc`、`.code-snippets`、`.jsonld`、`.geojson`、`.har`、`.webmanifest`、`.ipynb`、`.sarif` |
| `filenames[]` | `.prettierrc`、`.watchmanconfig`、`Pipfile.lock`、`deno.lock` |
| `content_types[]` | `application/json`、`application/jsonc` |

新增 suffixes 和 filenames 按表中顺序追加，不重排 Current entries。Exact filenames 保持现有 case-sensitive semantics。已由 `.json` 覆盖的 basename 不增加重复 exact entry。

### Decision 3: Every new match receives only generic structural navigation

任一新增 hint 命中后只选择既有 `docnav-json`。Selected operation 继续使用 Current grammar、logical tree、ref、outline、read、find、info、full-read、pagination、cost、diagnostic 与 output 契约。

Hint 命中不证明 profile validity。JSON-LD context expansion、GeoJSON geometry validation、HAR/Web Manifest/Notebook/SARIF schema semantics、lockfile version semantics 和 remote resolution 都需要独立需求、owner 和 change；它们不进入本 allowlist 扩展。

### Decision 4: Grammar and content-type ownership do not move

本 change 不根据 suffix、exact filename、declared content type 或 parser result 选择 grammar mode。所有新增 pathname 使用 Current JSONC-capable grammar；grammar-invalid input 返回 JSON-owned diagnostic 并保持 no-fallback。

Descriptor 继续精确声明 `application/json` 与 `application/jsonc`。本 change 只保留并验证这两个值，不新增、删除、推断或重新解释 descriptor/result content type，也不建立 pathname 到 profile、dialect 或 content type 的映射。

### Decision 5: Production implementation is manifest data only

Production diff 应只修改 built-in JSON manifest 的 `extensions[]` 和 `filenames[]` 单一 owner。Core derived indexes、normalization、lookup precedence 与 dispatch 继续从 manifest 数据工作，不增加 branch、probe、fallback 或 dependency。

`adapter list` 投影扩展后的 public manifest facts；automatic selection 内部消费 pathname，并把 unchanged closed standard operation input 交给 selected adapter。Protocol、CLI/env/config inputs、typed fields、invocation log、ref、continuation、content-type semantics 和 schema field shape 都不增加 matched-hint fact。

若 Current code 需要比 manifest data 更宽的 production diff，实施必须停止并重新审计本 design，而不是把 routing 变更吸收到本 change。

### Decision 6: Evidence separates selection from profile validity

Evidence 分为四层：

1. Manifest/registry assertions 精确证明完整有序集合、一个 `json` identity 与 unchanged content types。
2. Automatic-selection tests 逐一证明九个新增 pathname 选择 `docnav-json`。
3. 至少一个新增 suffix 和一个新增 exact filename 执行真实 `outline -> ref -> read`；代表性 invalid input 证明 selected parse failure 不 fallback。
4. CLI 与 release-package evidence 检查完整 manifest，并保留代表性 suffix/exact filename roundtrip。

测试复用 generic routing/navigation Cases，不为每个 domain profile 建立重复语义 corpus，因为本 change 不承诺 profile semantics。

### Decision 7: Exclusions remain closed

JSON5、NDJSON/JSONL、RFC 7464 JSON Text Sequences、模糊 rc names、弱 generic basenames、CBOR/BSON 等 binary JSON-like formats，以及任何 profile-specific navigation 均不加入。后续若支持任一排除项，必须先确定其 grammar/framing、document model、ref/continuation、diagnostic、output 和 validation owner。

## Risks and Responses

- **Misleading pathname：** 匹配文件可能损坏或不满足 profile。返回 Current JSON-owned parse diagnostic；文档和测试只声称 routing 与 generic navigation。
- **False semantic confidence：** 成功 outline 可能被误解为 profile validation。可观察文案不得使用“valid JSON-LD/GeoJSON/etc.”措辞。
- **Allowlist drift：** 多处复制 exact set 容易分叉。Manifest 是 production facts 的单一 owner；contract、tests 与 smoke 只声明或验证 public projection。
- **Compatibility change：** 原本得到 unsupported-format 的 pathname 可能变成 JSON-owned parse failure。这是本 change 的预期行为；回滚删除九个 hints 并同步对应证据。
- **Owner/current drift：** 长期 owner 或验证材料可能落后于 Current evidence。实施先在 task 1.1 恢复 pre-change baseline，再建立 Target；最终同步逐层核对 owner、main spec、code、tests、CLI 和 release evidence。

## Implementation Sequence

1. 完成 task 0：从 Current evidence 重建 delta，并通过 artifact audit。
2. 先同步长期 owner、主 spec 和 semantic Case 计划，再建立精确失败证据。
3. 只修改 manifest allowlist，使目标 tests 转绿。
4. 同步静态 projections、CLI/release evidence 和 Current 状态。
5. 运行目标验证、workspace verification 与最终 doubt-driven review；全部证据闭合后再评估归档。

回滚只删除九个新增 hints 和其专属 evidence，不需要数据迁移；调用者仍可通过 explicit `--adapter docnav-json` 使用 generic navigation。

## Open Questions

无未回答开放问题。实现、Current owner/spec 同步、开发与 release-package evidence 以及 workspace verification 已闭合；没有阻断 archive-readiness 评估的设计选择。
