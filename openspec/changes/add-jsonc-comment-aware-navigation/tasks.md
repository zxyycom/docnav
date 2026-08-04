本临时 task list 把 JSONC grammar、direct/tail comment attribution 与 `outline -> ref -> read` 作为一个 vertical slice 执行；task 0 是实现前阻塞门禁，不是产品实施。

## 0. Blocking implementation and artifact audit

**在0.1–0.7全部完成前，不得执行第1–6节，也不得修改Cargo manifest/lockfile、production code、owner docs、main specs、schemas/examples、fixtures、Case ledger、tests或release artifacts。** OpenSpec artifacts显示done/ready只证明文件齐全，不能替代该门禁。

- [ ] 0.1 从 `docs/navigation.md` 指向的 then-Current JSON/ref/protocol/output owners、main `json-adapter` spec、Current code/tests/release evidence 和相邻 active changes 恢复基线；在 `design.md` 只记录会改变 Decision 1–11 或 delta contract 的偏差，不把历史/active Target 当作 Current。
- [ ] 0.2 建立紧凑contract corpus：
  - 把已确认的`json:tail-comments:#<fragment>` virtual-entry package与root-tail merge规则纳入唯一README/design/delta/tasks contract，不保留待实现时选择的替代方案。
  - 逐项映射closed JSONC grammar、strict regression、root/object/array placement、empty-container-self attribution、tail-anchor boundaries、三种refs、conditional root entry、tail virtual entry、summary、三种read views、find mapping、source content type、diagnostics与large/deep/comment-heavy input。
  - 每个Case记录输入、expected observable result和owner requirement，不扩展到其它JSON-family产品语义。
- [ ] 0.3 用 workspace 外安全临时目录 spike Design Decision 9 的现实 parser/model candidates；证明 exact comment/token byte spans、raw numbers、decoded duplicates、depth `127/128`、source order、first-closer block comments、closed leniency controls、single-root diagnostics、bounded drop/work 与 current serializer compatibility，不修改 workspace dependency state。
- [ ] 0.4 比较候选的完整contract fit与总维护面，记录exact crate/version/features或exact custom scanner shape、direct/transitive dependency delta、license/advisory/maintenance、Rust/toolchain/target compatibility、representative build-size/startup/operation delta和rollback cost；只有一个候选满足完整contract时才在`design.md` Decision 9选择并链接point-in-time evidence。若全部no-fit，则记录blocking evidence并停止task 0，不能把“库能解析JSONC”或“最接近需求”当作comment-aware model已证明。
- [ ] 0.5 对已选实现与 delta spec 执行 bounded doubt-driven challenge：挑战 base ref compatibility、direct/tail ref coexistence与 stale behavior、attribution ambiguity、tail anchor stability、complete JSONC read grammar、virtual entry ordering、summary/pagination、comment find-to-read、selection-chain legal states、all-frame context preservation与 Current selected-view projection、source/content-type truthfulness、parser-default leakage、hostile input、document changes 和 downgrade；将 findings 分类并解决所有 contract gap/valid issue。
- [ ] 0.6 执行minimal implementation pass，确认已选方案只增加一个primary model、必要binding/anchor indexes、两个comment ref view markers与一条borrowed selected-first selection chain，不引入recursive parent tree、ancestor value clone、offset-based public comment identity、public dialect、shared protocol field、renderer parsing、per-item full comment scan或无当前消费者的JSON-family abstraction。
- [ ] 0.7 完成blocking artifact与AI-recovery audit：
  - 确认README/proposal/design/spec/tasks围绕同一句核心结果，`json-adapter` capability与delta path一致，Decision 1–11、attribution/ref/read contracts、private selection model和implementation selection无冲突，不存在未决product或architecture decision，本change未越权修改其它owner或active change。
  - 让未依赖本次对话的reader仅从README进入，并正确回答direct与tail归属差异、三种ref的identity关系、empty-container规则、root-tail grouping、selection frame必备facts、Current projection与private context上限，以及下一项允许执行的task；任一答案需要口头上下文即视为失败。
  - 运行strict OpenSpec validation、全部artifact `dnm outline`与docs/whitespace checks。只有审计证据记录完成后才可开始第1节。

## 1. Restore test evidence and establish failing behavior

- [ ] 1.1 读取 `docs/coding-style.md`、`docs/testing.md`、then-Current JSON behavior owner、`docs/testing/case-maintenance.md` 与项目 `test-evidence-review` skill；在任何测试修改前运行项目 wrapper，证明完整 Current static entities、runner entities 与 Case mapping 闭合。
- [ ] 1.2 为 strict/no-comment regression 和 JSONC vertical slice 新增或更新最小 semantic Cases，覆盖 corpus 中独立行为以及所有 renamed/split/merged test entities；重跑 Case completeness wrapper 后再改产品测试。
- [ ] 1.3 添加失败的loader/model tests，证明closed grammar、BOM/UTF-8、raw numbers、decoded duplicates、depth、source regions/comment spans、Decision 3 navigation-binding direct attribution、empty-container-self attribution、tail-anchor attribution、root internal/document tails合并为一个source-ordered bundle、absent与present-empty direct/tail bundles、slot mutual exclusivity和hostile bounded behavior。
- [ ] 1.4 添加失败的adapter operation与private ref-resolution tests，证明selected-first binding chain、root/empty-key/array-index区分、每个frame保留本层optional direct与tail bundles、Current read只投影selected frame/view、conditional root entry、tail virtual entry ordering、outline summary/paging、base/direct-comment/tail ref generation/resolution/coexistence/staleness、三种read content/type/cost/page、direct-comment/tail-comment/ordinary find-to-read和info/full-read source facts。
- [ ] 1.5 添加失败的 core/CLI/output/release tests，证明 descriptor facts、automatic/explicit one-grammar selection、selected failure no-fallback、opaque direct-comment/tail ref pass-through、schema-valid `protocol-json`、generic `readable-view` blocks 和 packaged binary behavior。

## 2. Synchronize owner contracts and validation materials

- [ ] 2.1 用最终delta更新`docs/adapters/json.md`与main `openspec/specs/json-adapter/spec.md`，把grammar、direct/tail attribution、三种ref views、outline/read/find、info/full-read、安全与diagnostics标为Target，保持shared owner只摘要opaque pass-through和既有result shape。
- [ ] 2.2 只在 task 0 证明 observable cross-owner fact 改变时同步 adapter/ref/protocol/output owner摘要；不得把 JSON grammar、attribution 或 ref parsing复制到 shared owner，也不得修改其它 adapter。
- [ ] 2.3 更新 manifest/schema/example/fixture/generated validation materials中受影响的既有字段和值；证明 `Entry.summary`、`ReadResult` 和 readable block仍使用 Current shape，不增加第二 schema source或 readable-only raw field。
- [ ] 2.4 运行 owner-specific Markdown、schema、example、fixture 和 contract-validation checks，并检查 scoped diff，确认所有 Target materials 对 ref spelling、content type、Current/Target 状态和非目标表达一致。

## 3. Implement parser, source model, and attribution

- [ ] 3.1 仅应用 task 0.4 选择的 dependency/version/features（如有），更新 workspace/adapter manifests和 lockfile一次；核对 resolved graph，若 license/security/feature/weight facts漂移则停止并返回 task 0。
- [ ] 3.2 更新唯一 `json_manifest()` owner，增加 `.jsonc` 和 `application/jsonc`，保持一个 `docnav-json` / `json` identity、Current hint order、closed input 与 no-probe behavior。
- [ ] 3.3 实现一个 JSONC-capable load path与单一 ordered source-aware logical tree，满足 selected-operation grammar、BOM/UTF-8、raw number、decoded duplicate、depth、source order、diagnostic normalization 和 bounded drop/work；显式关闭所有契约外leniency。
- [ ] 3.4 实现ordered non-overlapping comment spans、navigation-binding optional direct bundles与tail-anchor optional bundles，按Decision 3/delta placement rules做一次deterministic attribution pass；empty-container comments归container自身direct binding，root internal tail与document tail合并为一个source-ordered、可非连续的root bundle，`None`只表示absent，`Some`保证至少一个source-ordered index并允许normalized body为空字符串，每个comment恰好进入一个direct或tail slot；保留raw tokens且不暴露parser/CST types或建立第二棵full tree。
- [ ] 3.5 使 loader/model corpus与 hostile-input tests通过，并用 instrumentation或有界断言证明 outline/read/find不会为每个 item全量扫描 comment set。

## 4. Implement comment-aware navigation operations

- [ ] 4.1 把JSON ref handling分成syntax parse与document resolution：parser支持`json:comments:#<fragment>`与`json:tail-comments:#<fragment>` view markers和canonical tokens；resolver在primary document上产生borrowed selected-first selection chain，以显式root/object-member/array-element bindings保持empty key与comment三态，在每个frame保留binding/value/direct-bundle/tail-bundle context；实现direct-comment root/index、tail anchors、base compatibility、context-sensitive pointer validation、stale/no-comment `REF_NOT_FOUND`与malformed-view `REF_INVALID`，core/protocol继续原样传递。
- [ ] 4.2 扩展expanded-tree preorder outline projection和JSON entry pagination：有direct comments的navigation binding生成direct-comment ref与non-empty normalized `summary`，root container仅有root direct comments时在descendants前新增`<root>` entry，其它logical nodes保留base refs；每个tail-comment bundle在tail-anchor subtree末尾生成label `<tail comments>`、kind `tail_comments`、canonical tail ref与optional `summary`的virtual entry，并省略其它optional entry fields；预算先缩减summary，始终保留完整ref与可前进page。
- [ ] 4.3 从同一 `ResolvedSelection` 实现 base `application/json` read 与 Current direct-comment/tail `application/jsonc` projections：当前策略只选择首 frame与requested view，复用同一 normalized value serializer，按 source order拼接所选 exact comment tokens + LF，并对完整 projected content计算 cost后 Unicode-safe pagination；不得让 resolver丢弃 ancestor direct/tail context或把当前 projection shape变成 selection-model invariant。
- [ ] 4.4 扩展source find：完全位于direct-comment span的occurrence映射direct-comment ref，完全位于tail-comment span的occurrence映射tail-anchor ref，其它occurrence沿用Current deepest-covering positional mapping与base ref；保持source order、line location、bounded label和分页前进。
- [ ] 4.5 扩展 info/unstructured full-read 的 syntax-derived `application/json` / `application/jsonc` facts与 exact BOM-stripped source preservation，证明 string markers不误分类，format id保持 `json`。
- [ ] 4.6 统一所有 JSON/JSONC load、ref和 operation failures到contracted stable diagnostics，证明 parser messages/types、attribution internals、routing retry和adapter fallback不越界。

## 5. Targeted and cross-layer verification

- [ ] 5.1 运行 formatting、lint、targeted JSON unit/integration tests与完整 contract corpus；确认 strict snapshots不回归，所有 direct/tail attribution slots、三种ref/read views、virtual entry ordering、find mappings、content types、errors与 large-input bounds通过。
- [ ] 5.2 运行 core/navigation/CLI/protocol/output tests，验证 descriptor inspection、automatic/explicit selection、closed input、opaque refs、pagination/cost、schema-valid raw results、generic readable blocks和 selected-failure no-fallback。
- [ ] 5.3 运行 Case completeness/coverage wrapper、schema/example/fixture validators与 Linux/Windows canonical release-package smoke；确认 package core executable交付同一 linked behavior。
- [ ] 5.4 运行 `bun run verify:docnav-workspace`；调查每个 failure，直到通过或记录真实 external limitation、未验证 surface 与影响。

## 6. Final review and archive readiness

- [ ] 6.1 审查完整 diff 的 owner boundaries、one-change focus、dependency/model minimality、exact grammar/ref/content types、raw/readable parity、diagnostic privacy、downstream pathname-hint sequencing和 unrelated workspace preservation。
- [ ] 6.2 执行independent findings-first review与最终bounded doubt cycle，重点审查base/direct/tail ref compatibility、tail-anchor稳定性、attribution determinism、comment content validity、virtual entry ordering、pagination/cost、source fidelity、hostile input和rollback；解决所有actionable findings。
- [ ] 6.3 重跑 strict OpenSpec validation、所有改动 Markdown 的 `dnm outline`、docs/schema checks与 `git diff --check`；记录最终 evidence，确认 owner/spec/code/tests/release artifacts正确区分 Current 与 Target。
- [ ] 6.4 确认 downstream `expand-json-adapter-pathname-hints` 从本 change 已归档的 then-Current descriptor/grammar baseline重建其 delta，不把 comments或 parser语义带入 hint-only scope；随后请求本 change验收与归档。
