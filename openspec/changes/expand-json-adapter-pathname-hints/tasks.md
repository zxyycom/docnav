本临时 task list 只在 JSONC predecessor 已由当前实现证据证明为 Current 且 task 0 审计通过后，才允许实施强 JSON-family pathname hint allowlist 与同步证据；实施必须完整保留 predecessor 的 `application/json` / `application/jsonc` descriptor content-type set。

## 0. Blocking predecessor, current-baseline, and artifact audit

Tasks 0.1–0.5 仅允许只读调查和修改本 change artifacts。**在 0.6 明确完成前，不得开始 sections 1–5，也不得修改 production code、owner docs、main specs、schemas、examples、fixtures、Case ledger、tests、release materials、Cargo files 或其它 change。OpenSpec `applyRequires` 状态为 done 不满足此门禁。**

- [ ] 0.1 证明 `support-jsonc-in-json-adapter` 已完成其自身 blocking gates、owner 同步、实现和范围匹配验证，并由 then-Current code、tests 与 release evidence（而非 active/archived change 状态或勾选任务本身）证明一个 `json` identity、`.json` / `.code-workspace` / `.jsonc` suffixes、exact `.prettierrc` / `.watchmanconfig` filenames、`application/json` / `application/jsonc` descriptor content types 和 JSONC grammar 均为 Current；若任一项未满足，停止本 change 实施并记录 blocking evidence。
- [ ] 0.2 从 `docs/navigation.md` 进入并重读 adapter contract、JSON adapter、coding style、testing、Case maintenance 和 `test-evidence-review` skill；运行 `bun run test-evidence -- check --root .` 证明完整 current tree 的 static/runtime/Case 映射闭合，随后只读盘点 then-Current manifest、registry/listing、automatic selection、CLI/release smoke 与 schema/example evidence。
- [ ] 0.3 记录 then-Current `docnav-json` 的 exact adapter/format/content-type facts、`extensions[]` / `filenames[]` 完整有序值和其唯一 production owner；精确核对 descriptor 仍声明 `application/json` 与 `application/jsonc`，matched content type 不进入 strategy input，`.json` 已覆盖的 basenames、新增七个 suffixes 与两个 exact filenames、所有排除项，以及 Current routing/explicit/no-fallback owner，确认 implementation candidate 只需 manifest allowlist 数据与同步证据。
- [ ] 0.4 从 then-Current `openspec/specs/json-adapter/spec.md` 复制 `JSON adapter 必须作为静态 linked adapter 提供` 的完整 requirement（正文和全部 scenarios），再重建本 change 的 `MODIFIED` block：保留 predecessor 的实际内容、顺序、一个 `json` identity、`application/json` / `application/jsonc` descriptor values 和 matched-content-type input exclusion，按 Decision 2 只追加九个 hints，不保留当前草案对 `.jsonc` 位置或 predecessor wording 的任何未经证明假设；同步修正 proposal/design/tasks 中发现的基线 drift。
- [ ] 0.5 对 exact allowlist、generic-read boundary、一个 adapter/format/grammar identity、unchanged descriptor/result content-type ownership、hint-only selection、invalid selected input、protocol/process boundary、profile/remote-resolution exclusions、ordering、rollback、test evidence 和 cross-change ownership 做 bounded doubt-driven challenge；将 substantive findings 归类并修正 artifacts，任何未解决 contract gap 或 valid issue 都继续阻塞。
- [ ] 0.6 **Blocking artifact/current-baseline audit:** 确认 proposal、design、delta spec、tasks 与 README 都以同一“predecessor 后只扩展强 hints、保留两个 descriptor content types、保持 generic navigation”核心句和临时 artifact 身份开头；capability id 与 delta directory 精确复用 `json-adapter`；注册 requirement 是 exact hint set 的唯一 delta owner，且由 then-Current 完整重建；九个新增项、现有/predecessor 项、`application/json` / `application/jsonc` descriptor values、matched-content-type input exclusion、Non-Goals、numbered Decisions、protocol/process effects、evidence matrix 和 rollback 相互一致；没有 artifact 声称 main spec 当前已含 `.jsonc` / `application/jsonc`、实现已批准或该能力已 Current，也没有 artifact 让本 change 新增或解释 content type；`## Open Questions` 无未回答问题；task 0 未修改本 change 外文件；`openspec validate expand-json-adapter-pathname-hints --type change --json --strict --no-interactive` 通过；每个 Markdown 均可由 `bun --silent run dnm outline` 读取。**只有全部检查通过并勾选 0.6 后，sections 1–5 才可开始。**

## 1. Establish owner and failing evidence

- [ ] 1.1 依据已审计 delta 同步 `docs/adapters/json.md` 的 pathname hint owner 表述与 `openspec/specs/json-adapter/spec.md` 的完整注册 requirement；保留 predecessor 的两个 descriptor content types 和 JSON adapter owner 对 result content type 的解释，保留共享 routing algorithm 的引用而不复制它，并在实现证据尚未通过时不把新增集合标为 Current。
- [ ] 1.2 用项目 wrapper 的 `topics` / bounded `list` / `show` 定位 `json-adapter` 与 `core-cli` 现有 Cases；先写清 owner promise 到 observable result，再决定复用或新增 Case，并在改测试前同步 Case 内容/实体计划，避免为每个 profile 建立重复语义 Case。
- [ ] 1.3 增加或更新 JSON adapter manifest 与 core registry/listing 的 table-driven assertions，使其精确覆盖 then-Current entries、七个新增 suffixes、两个新增 exact filenames、稳定顺序、一个 `json` identity、unchanged `application/json` / `application/jsonc` descriptor facts、matched content type 不进入 strategy input 和 unchanged public input inventory；同步 protocol/schema fixture 的 manifest 值断言但不改变 schema shape。
- [ ] 1.4 增加 table-driven automatic-selection tests，逐一证明每个新增 suffix/exact filename 选择 `docnav-json`；至少为一个 suffix 和一个 exact filename 执行真实 `outline -> ref -> read`，并用 representative grammar-invalid content 证明 hint 不是 validity assertion 且 selected failure 不 reroute/fallback。
- [ ] 1.5 扩展真实 core CLI smoke 和 release-package smoke/inspection expectations：`adapter list` 覆盖完整 manifest set 与 unchanged `application/json` / `application/jsonc` descriptor values，开发 binary 与 packaged binary 各保留代表性新增 suffix/exact filename routing roundtrip；不加入 content-type 推断、profile-validity、domain semantics 或 remote resolution assertions。
- [ ] 1.6 运行能独立报告上述目标实体的最窄 adapter/core/smoke commands，确认新增 assertions 因旧 allowlist 精确失败且既有 JSONC、generic navigation、explicit selection、diagnostic 和 public-input assertions 仍通过；若失败暴露其它 contract gap，返回 task 0 而不是放宽预期。

## 2. Implement the minimal manifest change

- [ ] 2.1 在 then-Current built-in JSON manifest 的单一 `extensions[]` owner 中保留原 entries 原顺序，并依次追加 `.code-snippets`、`.jsonld`、`.geojson`、`.har`、`.webmanifest`、`.ipynb`、`.sarif`。
- [ ] 2.2 在同一 format descriptor 的 `filenames[]` 中保留原 entries 原顺序，并依次追加 `Pipfile.lock`、`deno.lock`；保持 exact-filename case semantics、adapter id `docnav-json`、format id `json`、`application/json` / `application/jsonc` descriptor content-type set 与 capabilities 不变。
- [ ] 2.3 用局部 diff 确认 production change 只有 manifest allowlist data：没有新增 dependency、parser/profile branch、routing algorithm、probe/fallback、public input、protocol/schema field、ref/output/content-type 或 continuation change；若 Current code 需要更广 production diff，停止并回到 design 审计。
- [ ] 2.4 运行 JSON adapter 与 core registry/selection 的目标 Rust tests，证明全部 exact-set、unchanged two-content-type descriptor、selection、roundtrip、invalid-no-fallback 和 unchanged-input assertions 转绿。

## 3. Close synchronized documentation and test evidence

- [ ] 3.1 在目标 tests 确认行为后，把 JSON owner 文档的完整 hint table、generic-navigation boundary、validation wording 与 OpenSpec main spec 同步为 Current；确保 `.jsonc` grammar 和 descriptor/result content-type 所有权仍指向已落地 predecessor 与 JSON adapter owner，本 change 不重复完整 routing algorithm 或解释 content type。
- [ ] 3.2 按 `docs/testing/case-maintenance.md` 和 `test-evidence-review` 审查每个变化测试的 owner、observable signal、reliability、independence 与维护价值；更新 `json-adapter` / `core-cli` Case mappings 和 coverage materials，复用共同 generic-routing Case，不为 fixture/build step 创建名义 Case。
- [ ] 3.3 同步所有静态 manifest projections、registry/listing snapshots、schema/example value fixtures 与 CLI/release inspection expectations；明确 pathname arrays 是唯一改变的 values，`application/json` / `application/jsonc` descriptor set 只被保留和断言，manifest/schema/protocol shape 与 result content-type 语义不变。
- [ ] 3.4 运行 `bun run test-evidence -- check --root .`，证明修改后的完整 static/runtime/Case 集合双向闭合；处理任何未知、悬空、重复或语义失配实体，不用生成模板 Case 消除诊断。

## 4. Targeted and workspace verification

- [ ] 4.1 运行 Rust formatting 和目标 package checks/tests（至少覆盖 `docnav-json`、core `docnav` registry/selection 与 shared protocol manifest fixtures），保存精确通过范围并确认无 warning 被误报为通过。
- [ ] 4.2 运行 `bun run smoke:docnav`，验证开发 binary 的 full manifest inspection（包括两个 unchanged descriptor content types）、代表性新增 suffix/exact filename `outline -> ref -> read` 和 selected invalid no-fallback 行为。
- [ ] 4.3 通过项目 release-package build/verify/smoke 流程验证 canonical packaged `docnav` binary 的同一 manifest facts 和代表性 roundtrips；若目标 package/host 不可用，记录未验证边界而不宣称 release behavior Current。
- [ ] 4.4 先运行 `bun run verify:docnav-workspace:required` 做快速闭合，再运行 `bun run verify:docnav-workspace` 覆盖 Rust、CLI smoke、OpenSpec、docs、schema/examples、Case ledger、clippy 和 release-related checks；修复范围内 failures，并单独报告任何外部 baseline failure。
- [ ] 4.5 运行 `openspec validate expand-json-adapter-pathname-hints --type change --json --strict --no-interactive`，并对本 change 的 README、proposal、design、spec 与 tasks 分别运行 `bun --silent run dnm outline`。

## 5. Final contract and archive-readiness review

- [ ] 5.1 执行 coding-style 变更前后自检与局部 diff review，证明一个 manifest owner、一个 `json-adapter` contract owner、两个 predecessor-owned descriptor content types、无 duplicated routing/content-type algorithm、无 profile semantics、无 remote resolution、无 excluded formats、无 dependency 或 public-surface expansion。
- [ ] 5.2 将最终实现、owner docs、main spec、Cases、tests、schema/example fixtures、CLI/release evidence 与本 delta 逐项比对；对 strict JSON、JSONC、unchanged descriptor content-type set 和每个新增 hint 的事实分别标明 Current evidence，不把 change/历史记录本身当作实现证明。
- [ ] 5.3 重做 bounded doubt-driven review，重点挑战 misleading-path failure、case-sensitive exact filenames、ASCII-normalized suffixes、predecessor descriptor/content-type regression、hint ordering、no-fallback、content-type/profile inference、profile-invalid success/parse failure 和 rollback；解决所有 substantive findings。
- [ ] 5.4 确认所有 tasks 与验证证据真实完成后再评估归档；归档前从最终 Current main requirement 重核 `MODIFIED` block，且未经单独授权不执行 archive、release、commit、push 或其它外部状态变更。
