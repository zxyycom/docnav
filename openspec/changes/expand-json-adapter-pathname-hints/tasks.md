本 task list 交付一个 manifest-only 的 pathname-hint 扩展。执行顺序固定为：恢复 Current JSONC 基线并审计 artifacts，建立 owner 与失败证据，只修改 manifest 数据，最后闭合文档、测试、CLI 和 release evidence。任一 blocking task 失败时停止，不扩大 production scope。

## 0. Current baseline and artifact gate

- [x] 0.1 从 `docs/navigation.md` 进入，读取 adapter contract、JSON adapter owner、main `json-adapter` spec、coding style、testing、Case maintenance 和 `test-evidence-review` skill；区分长期契约、Current 实现证据和 OpenSpec 历史。
- [x] 0.2 运行 `bun run test-evidence -- check --root .`；盘点 Current `json_manifest()`、registry/listing、automatic/explicit selection、JSONC grammar、CLI/release smoke 与 schema/example evidence，确认 Current exact facts 为一个 `docnav-json` / `json` identity、`.json` / `.code-workspace` / `.jsonc`、`.prettierrc` / `.watchmanconfig`、`application/json` / `application/jsonc` 和统一 JSONC-capable grammar。
- [x] 0.3 从 Current main spec 完整重建本 change 的 `MODIFIED` requirement，保留正文和全部 Current scenarios，再只追加 Decision 2 的九个 pathname hints 与 generic-navigation scenario；不得把 `.jsonc`、JSONC grammar 或 `application/jsonc` 记为本 change 新增。
- [x] 0.4 对 allowlist、一个 adapter/format/grammar identity、generic-navigation boundary、content-type ownership、invalid selected input、no-fallback、public input、ordering、排除项、rollback 和 cross-change ownership 做 bounded doubt-driven review；修复所有 substantive findings。
- [x] 0.5 **Blocking artifact audit:** README、proposal、design、delta spec 与 tasks 必须对 Purpose、Current/Target、exact ordered sets、owner、implementation surface、observable effects 和验证层级表达一致；`## Open Questions` 无未回答问题；task 0 不修改本 change 目录外文件；strict OpenSpec validation、全部 change Markdown 的 `dnm outline` 与 `git diff --check` 通过。只有勾选本项后才可执行 sections 1–4。

## 1. Establish owner contract and failing evidence

- [ ] 1.1 先把 `docs/adapters/json.md` 的 Current 基线与 Current code、tests 和 release evidence 对齐，再写入本 change 的 Target hint table、hint-only selection 与 generic-navigation boundary；同步 main `openspec/specs/json-adapter/spec.md` 的完整注册 requirement，不复制 shared routing algorithm。
- [ ] 1.2 按 `docs/testing/case-maintenance.md` 和 `test-evidence-review` 恢复 `json-adapter` / `core-cli` Cases；为 manifest exact set、automatic selection、representative roundtrip 和 invalid-no-fallback 选择最小独立 Cases，不为每个 domain profile 建立重复 Case。
- [ ] 1.3 增加 table-driven manifest/registry/listing assertions：覆盖完整有序 `extensions[]` / `filenames[]`、一个 `json` identity、unchanged `application/json` / `application/jsonc`、matched metadata 不进入 strategy input 和 unchanged public input inventory。
- [ ] 1.4 增加 table-driven automatic-selection tests，逐一证明九个新增 pathname 选择 `docnav-json`；至少一个 suffix 和一个 exact filename 执行真实 `outline -> ref -> read`，代表性 grammar-invalid input 证明 selected failure 不 reroute/fallback。
- [ ] 1.5 扩展开发 CLI 与 release-package smoke expectations：`adapter list` 覆盖完整 manifest，开发 binary 与 packaged binary 各保留代表性 suffix/exact-filename roundtrip；运行最窄目标命令，确认新 assertions 因旧 allowlist 精确失败，既有 JSONC、generic navigation、explicit selection 和 diagnostics 仍通过。

## 2. Implement the manifest-only change

- [ ] 2.1 在 built-in JSON manifest 的单一 `extensions[]` owner 中保留 Current entries 和顺序，再依次追加 `.code-snippets`、`.jsonld`、`.geojson`、`.har`、`.webmanifest`、`.ipynb`、`.sarif`。
- [ ] 2.2 在同一 descriptor 的 `filenames[]` 中保留 Current entries 和顺序，再依次追加 `Pipfile.lock`、`deno.lock`；保持 exact-filename case semantics、adapter/format identity、capabilities 与两个 descriptor content types 不变。
- [ ] 2.3 用局部 diff 证明 production change 只有 manifest allowlist data，并运行目标 JSON adapter、core registry/selection 与 smoke tests 使 section 1 的失败证据转绿。若需要新增 dependency、parser/profile branch、routing/fallback、public input、protocol/schema field、ref/output/content-type 或 continuation change，停止并返回 design 审计。

## 3. Synchronize Current evidence and run verification

- [ ] 3.1 在目标 tests 通过后，把 JSON owner 文档和 main spec 的 hint table 标为 Current；同步 semantic Cases、coverage、static manifest projections、schema/example value fixtures 与 CLI/release expectations，明确只有 pathname arrays 的 values 改变。
- [ ] 3.2 按 `test-evidence-review` 检查每个变化测试的 owner、observable signal、independence 与维护价值；运行 `bun run test-evidence -- check --root .`，证明完整 static/runtime/Case 映射闭合。
- [ ] 3.3 运行 Rust formatting、target package checks/tests 与 `bun run smoke:docnav`，验证完整 manifest、representative roundtrips 和 selected invalid no-fallback。
- [ ] 3.4 通过项目 release-package build/verify/smoke 流程验证 packaged `docnav` 的同一 manifest facts 和 representative roundtrips；目标 host 不可用时记录未验证边界，不宣称该 release surface Current。
- [ ] 3.5 先运行 `bun run verify:docnav-workspace:required`，再运行 `bun run verify:docnav-workspace`；另外运行 strict OpenSpec validation、全部 change Markdown 的 `dnm outline` 和 `git diff --check`。

## 4. Final contract and archive-readiness review

- [ ] 4.1 执行 coding-style 自检与局部 diff review，证明一个 manifest owner、一个 `json-adapter` contract owner、一个 adapter/format/grammar identity、两个 unchanged descriptor content types，且没有 profile、remote resolution、dependency 或 public-surface expansion。
- [ ] 4.2 将最终实现、owner docs、main spec、Cases、tests、schema/examples、CLI 和 release evidence 与本 delta 逐项比对；分别标明 Current JSONC baseline 与九个新增 hints 的实现证据，不把 OpenSpec artifact 或任务勾选本身当作 Current 证明。
- [ ] 4.3 重做 bounded doubt-driven review，重点挑战 misleading pathname、case-sensitive exact filenames、suffix normalization、hint ordering、content-type/profile inference、grammar-invalid input、no-fallback 与 rollback；解决所有 substantive findings。
- [ ] 4.4 所有 tasks 与验证证据真实完成后再评估归档。归档前从最终 Current main requirement 重核完整 `MODIFIED` block；未经单独授权不执行 archive、release、commit、push 或其它外部状态变更。
