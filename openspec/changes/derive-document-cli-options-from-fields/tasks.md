## 1. 决策记录与实现基线

- [x] 1.1 记录已确认边界：同一 operation 的 registry public flags 本期全局唯一，同名复用留给后续 change；field projection 不枚举 `output` mode 集合，并行 output change 不构成实施顺序依赖。
- [ ] 1.2 按 `docs/navigation.md` 入口读取 coding、testing 与相关 owner materials，记录 common/native declarations、Clap projection、catalog、lexical preflight、raw bridge 和 handler binding 的当前调用关系、现有证明与保留/删除边界。
- [ ] 1.3 先更新 `docs/architecture.md`、`docs/cli.md`、`docs/navigation-input-resolution.md`、`docs/adapter-contract.md` 及 typed-fields/CLI-resolution package READMEs，标明 Target 与 Current owner boundary。

## 2. Canonical CLI metadata

- [ ] 2.1 在 typed-fields 中增加 framework-neutral CLI processing metadata，覆盖 help、value name、valueless Boolean switch 与 explicit token-to-Boolean mapping，并复用既有 CLI locator 和 canonical field facts。
- [ ] 2.2 增加 typed-fields contract tests，证明 metadata 经过 builder clone、declaration type erasure、field build 和 `FieldDefSet` aggregation 后保留，并确定性拒绝 invalid attachment、duplicate metadata、incompatible value kind 和 incomplete Boolean mapping。

## 3. Clap direct projection

- [ ] 3.1 扩展 `cli-config-resolution-clap` argument/help projection，从 canonical metadata 构造 identity、flag、help/value name、accepted/default display、cardinality 与 Boolean capture，并拒绝 `ValueKind::Json`、ambiguous locator 和 static conflict。
- [ ] 3.2 从 successful `ArgMatches` 构造 normalized CLI `Source`，支持 string、integer、finite number、两类 Boolean encoding、repeated string array 和 repeated `key=value` object；decode failure 保留为 field-local invalid candidate，omitted/default 不形成 explicit candidate。
- [ ] 3.3 增加 companion tests，覆盖 help derivation、Boolean encodings、numeric capture、array/object cardinality、raw/reason preservation、unrelated candidate continuation、structural failures 和 canonical identity preservation。

## 4. Docnav declarations 与 field sets

- [ ] 4.1 让 common declarations author `adapter`、`page`、`limit`、`pagination` 和 `output` 的 CLI metadata；`output` projection 只读取 canonical accepted/default facts。
- [ ] 4.2 让 `AdapterOptionSpec` 复用同一 CLI metadata，并迁移 built-in adapter native options；config-only option 保持不生成 public flag。
- [ ] 4.3 构造 operation-scoped registry CLI `FieldDefSet`，按 common、registry、adapter declaration order 聚合 fields，并对 same-operation flag/static conflicts 返回带 owner/field attribution 的 declaration failure。
- [ ] 4.4 Adapter selection 后构造 selected `FieldDefSet`，用 contract tests 证明 registry/selected sets 复用相同 identity、locator、value kind、constraints、default 和 applicability facts。

## 5. Core/navigation cutover

- [ ] 5.1 Core 用 static document command shape 加 registry CLI field set 生成 Clap command/help；保留的 lexical preflight 从 static shape 与同一 projection 派生 document flag/cardinality facts。
- [ ] 5.2 将 fixed positional facts、normalized CLI `Source`、config descriptors/paths 和 registry 交给 navigation；routing 完成 adapter selection 后由 selected set 执行 existing priority、merge、validation 和 typed materialization。
- [ ] 5.3 对 selected invalid candidate 保持 canonical blocking；对不属于 selected adapter/current-operation set 的 explicit candidate 返回 strict unsupported/unused diagnostic并停止 dispatch。
- [ ] 5.4 在 projection、selection 和 regression tests 通过后删除 `NativeOptionCatalog`、derived arg-id table、raw native strings、post-parse flag lookup、JSON guessing、parallel decoder 和 runtime fallback；helper 仅在调用方清零时删除。
- [ ] 5.5 增加 core/navigation/process tests，覆盖 common/native flags、operation-scoped help、declaration conflicts、unknown/duplicate/missing value、selected invalid、unselected explicit failure 和 typed handler handoff。

## 6. Verification 与交付

- [ ] 6.1 按 `docs/testing/case-maintenance.md` 更新 case materials、源码 `@case` 标记和 coverage mapping；仅在实际可观察 contract 变化时更新 schema、examples、fixtures 或 release artifacts。
- [ ] 6.2 运行 `cargo fmt --all --check`，并对 touched packages 运行范围匹配的 all-target tests、doc tests、examples、`cargo check` 和 clippy `-D warnings`。
- [ ] 6.3 运行 targeted CLI/process smoke、case consistency、docs validation、`bun run verify:docnav-workspace` 和 strict OpenSpec validation，要求所有适用检查 0 failed。
- [ ] 6.4 搜索 runtime document option path 的重复 semantics、catalog、derived arg ids、raw strings、JSON guessing 和 fallback；确认保留 helper 有调用方，并按 `docs/coding-style.md` 完成局部 diff 与交付审计。
