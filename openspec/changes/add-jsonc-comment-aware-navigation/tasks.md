本 task list 把 JSONC grammar、direct/tail comment attribution 与 `outline -> ref -> read` 作为一个 vertical slice 执行。Task 0 的实施前审计已完成，证据统一记录在 [`design.md`](design.md#implementation-audit)；产品实施从 task 1.1 开始，其余未完成任务仍保持未勾选。

## 0. Implementation-readiness audit (complete)

- [x] 0.1 恢复 `docs/navigation.md` 指向的 Current JSON/ref/protocol/output owners、main spec、code/tests/release evidence 和相邻 active-change baseline，未把 historical/active Target 当作 Current。
- [x] 0.2 建立闭合 contract corpus，映射 grammar、attribution、三种 ref/read views、outline/find、source facts、diagnostics、integration 与 bounded-input evidence，未扩展其它 JSON-family 产品语义。
- [x] 0.3 在 workspace 外隔离 temp crate 中 spike 三个现实 parser/model 候选；Rust 1.96.0 下 10 个针对性用例通过，workspace dependency state 未改变。
- [x] 0.4 按完整 contract fit 与总维护面选择 private offset-preserving scanner + Current `serde_json 1.0.150`；记录候选版本、差异、dependency/license/advisory/toolchain/target、size/latency 与 rollback evidence。
- [x] 0.5 完成 bounded doubt-driven challenge，收敛单行 summary、按需派生 bundle 文本和 unique comment-ref auto-read evidence，其余 ref/attribution/read/context/source/bounds 问题无未解决 contract gap。
- [x] 0.6 完成 minimal implementation pass：保留一棵 primary tree、一个 load-time 等长 buffer、必要 comment/binding/anchor indexes、两个 comment view markers 与一条 borrowed selection chain，不增加无消费者的 abstraction 或 public surface。
- [x] 0.7 完成 artifact/AI-recovery audit：README/proposal/design/delta/tasks 的 owner 与核心结果一致，reader 可从 README 恢复必要决策与下一任务，strict OpenSpec、`dnm outline`、docs 与 whitespace validation 通过。

## 1. Restore test evidence and establish failing behavior

- [x] 1.1 读取 `docs/coding-style.md`、`docs/testing.md`、then-Current JSON behavior owner、`docs/testing/case-maintenance.md` 与项目 `test-evidence-review` skill；在任何测试修改前运行项目 wrapper，证明完整 Current static entities、runner entities 与 Case mapping 闭合。
- [x] 1.2 为 strict/no-comment regression 和 JSONC vertical slice 新增或更新最小 semantic Cases，覆盖 corpus 中独立行为以及所有 renamed/split/merged test entities；重跑 Case completeness wrapper 后再改产品测试。
- [x] 1.3 添加失败的loader/model tests，证明closed grammar、BOM/UTF-8、raw numbers、decoded duplicates、depth、source regions/comment spans、Decision 3 navigation-binding direct attribution、empty-container-self attribution、tail-anchor attribution、root internal/document tails合并为一个source-ordered bundle、absent与present-empty direct/tail bundles、slot mutual exclusivity和hostile bounded behavior。
- [x] 1.4 添加失败的adapter operation与private ref-resolution tests，证明selected-first binding chain、root/empty-key/array-index区分、每个frame保留本层optional direct与tail bundles、Target read只投影selected frame/view、conditional root entry、tail virtual entry ordering、outline summary/paging、base/direct-comment/tail ref generation/resolution/coexistence/staleness、三种read content/type/cost/page、direct-comment/tail-comment/ordinary find-to-read和info/full-read source facts。
- [x] 1.5 添加失败的 core/CLI/output/release tests，证明 descriptor facts、automatic/explicit one-grammar selection、selected failure no-fallback、opaque direct-comment/tail ref pass-through、unique direct/tail ref 的 existing auto-read、schema-valid `protocol-json`、generic `readable-view` nested blocks 和 packaged binary behavior。

## 2. Synchronize owner contracts and validation materials

- [x] 2.1 用最终delta更新`docs/adapters/json.md`与main `openspec/specs/json-adapter/spec.md`，把grammar、direct/tail attribution、三种ref views、outline/read/find、info/full-read、安全与diagnostics标为Target，保持shared owner只摘要opaque pass-through和既有result shape。
- [x] 2.2 只在 task 0 证明 observable cross-owner fact 改变时同步 adapter/ref/protocol/output owner摘要；不得把 JSON grammar、attribution 或 ref parsing复制到 shared owner，也不得修改其它 adapter。
- [x] 2.3 更新 manifest/schema/example/fixture/generated validation materials中受影响的既有字段和值；证明 `Entry.summary`、`ReadResult` 和 readable block仍使用 Current shape，不增加第二 schema source或 readable-only raw field。
- [x] 2.4 运行 owner-specific Markdown、schema、example、fixture 和 contract-validation checks，并检查 scoped diff，确认所有 Target materials 对 ref spelling、content type、Current/Target 状态和非目标表达一致。

## 3. Implement parser, source model, and attribution

- [x] 3.1 确认 task 0.4 选定路径不新增 dependency、feature 或 lockfile entry；descriptor manifest 只修改 task 3.2 的既有 facts。若实现被迫重开 parser/dependency 选择，停止并重新执行 task 0.3–0.6，不在本 task 中顺带引入。
- [x] 3.2 更新唯一 `json_manifest()` owner，增加 `.jsonc` 和 `application/jsonc`，保持一个 `docnav-json` / `json` identity、Current hint order、closed input 与 no-probe behavior。
- [x] 3.3 在 private `jsonc` lexical module 实现一次线性扫描、等长 neutralized parse view 与 selected-operation closed grammar；让 Current `serde_json 1.0.150` `NodeSeed`/`BuildState` 复用同一 offsets 构建唯一 ordered source-aware logical tree，保留 BOM/UTF-8、raw number、decoded duplicate、depth、source order、diagnostic normalization 和 bounded drop/work。
- [x] 3.4 实现ordered non-overlapping comment spans、navigation-binding optional direct bundles与tail-anchor optional bundles，按Decision 3/delta placement rules做一次deterministic attribution pass；empty-container comments归container自身direct binding，root internal tail与document tail合并为一个source-ordered、可非连续的root bundle，`None`只表示absent，`Some`保证至少一个source-ordered index，summary/read 按需从 source spans 派生且不缓存第二份 normalized text，每个comment恰好进入一个direct或tail slot；保留raw tokens且不暴露parser/CST types或建立第二棵full tree。
- [x] 3.5 使 loader/model corpus与 hostile-input tests通过，并用 instrumentation或有界断言证明 outline/read/find不会为每个 item全量扫描 comment set。

## 4. Implement comment-aware navigation operations

- [x] 4.1 把JSON ref handling分成syntax parse与document resolution：parser支持`json:comments:#<fragment>`与`json:tail-comments:#<fragment>` view markers和canonical tokens；resolver在primary document上产生borrowed selected-first selection chain，以显式root/object-member/array-element bindings保持empty key与comment三态，在每个frame保留binding/value/direct-bundle/tail-bundle context；实现direct-comment root/index、tail anchors、base compatibility、context-sensitive pointer validation、stale/no-comment `REF_NOT_FOUND`与malformed-view `REF_INVALID`，core/protocol继续原样传递。
- [x] 4.2 扩展expanded-tree preorder outline projection和JSON entry pagination：有direct comments的navigation binding生成direct-comment ref与按需派生的单行 normalized `summary`（多 body 用 `; ` 连接），root container仅有root direct comments时在descendants前新增`<root>` entry，其它logical nodes保留base refs；每个tail-comment bundle在tail-anchor subtree末尾生成label `<tail comments>`、kind `tail_comments`、canonical tail ref与optional `summary`的virtual entry，并省略其它optional entry fields；预算先缩减summary，始终保留完整ref与可前进page。
- [x] 4.3 从同一 `ResolvedSelection` 实现 base `application/json` read 与 Target direct-comment/tail `application/jsonc` projections：本 change 的策略只选择首 frame与requested view，复用同一 normalized value serializer，按 source order拼接所选 exact comment tokens + LF，并对完整 projected content计算 cost后 Unicode-safe pagination；不得让 resolver丢弃 ancestor direct/tail context或把本 projection shape变成 selection-model invariant。
- [x] 4.4 扩展source find：完全位于direct-comment span的occurrence映射direct-comment ref，完全位于tail-comment span的occurrence映射tail-anchor ref，其它occurrence沿用Current deepest-covering positional mapping与base ref；保持source order、line location、bounded label和分页前进。
- [x] 4.5 扩展 info/unstructured full-read 的 syntax-derived `application/json` / `application/jsonc` facts与 exact BOM-stripped source preservation，证明 string markers不误分类，format id保持 `json`。
- [x] 4.6 统一所有 JSON/JSONC load、ref和 operation failures到contracted stable diagnostics，证明 parser messages/types、attribution internals、routing retry和adapter fallback不越界。

## 5. Targeted and cross-layer verification

- [x] 5.1 运行 formatting、lint、targeted JSON unit/integration tests与完整 contract corpus；确认 strict snapshots不回归，所有 direct/tail attribution slots、三种ref/read views、virtual entry ordering、find mappings、content types、errors与 large-input bounds通过。
- [x] 5.2 运行 core/navigation/CLI/protocol/output tests，验证 descriptor inspection、automatic/explicit selection、closed input、opaque refs、unique direct/tail ref auto-read、pagination/cost、schema-valid raw results、generic readable base/nested blocks和 selected-failure no-fallback。
- [x] 5.3 运行 Case completeness/coverage wrapper、schema/example/fixture validators与 Linux/Windows canonical release-package smoke；确认 package core executable交付同一 linked behavior。
- [x] 5.4 运行 `bun run verify:docnav-workspace`；调查每个 failure，直到通过或记录真实 external limitation、未验证 surface 与影响。

## 6. Final review and archive readiness

- [x] 6.1 审查完整 diff 的 owner boundaries、one-change focus、dependency/model minimality、exact grammar/ref/content types、raw/readable parity、diagnostic privacy、downstream pathname-hint sequencing和 unrelated workspace preservation。
- [x] 6.2 执行independent findings-first review与最终bounded doubt cycle，重点审查base/direct/tail ref compatibility、tail-anchor稳定性、attribution determinism、comment content validity、virtual entry ordering、pagination/cost、source fidelity、hostile input和rollback；解决所有actionable findings。
- [x] 6.3 重跑 strict OpenSpec validation、所有改动 Markdown 的 `dnm outline`、docs/schema checks与 `git diff --check`；记录最终 evidence，确认 owner/spec/code/tests/release artifacts正确区分 Current 与 Target。
- [x] 6.4 确认 downstream `expand-json-adapter-pathname-hints` 从本 change 已归档的 then-Current descriptor/grammar baseline重建其 delta，不把 comments或 parser语义带入 hint-only scope；随后请求本 change验收与归档。
