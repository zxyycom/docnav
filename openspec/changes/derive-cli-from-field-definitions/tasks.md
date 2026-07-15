## 1. 目标文档、证据与契约基线

- [ ] 1.1 先更新 `docs/navigation.md`、`docs/architecture.md`、`docs/cli.md`、`docs/output.md`、`docs/navigation-input-resolution.md`、`docs/adapter-contract.md`、typed-fields README 与两个 `cli-config-resolution` README；immutable extension、projection bridge、CLI structural/candidate 分层、stage-scoped config、selected-field 和 output-context 规则分别只在其 owner 文档完整定义，并按 `docs/navigation.md` 的状态语义区分目标规范与 Current 实现。
- [ ] 1.2 依据更新后的 owner docs 审计 typed-fields builder/declaration erasure、`OperationFieldSet`、`AdapterOptionSpec`、`cli-config-resolution-clap`、resolver invalid-candidate behavior、core-owned logging config、routing/outline/selected config stages、`config inspect`、core Clap builders、quality accepted warnings 和 legacy scanners；在 `design.md` 的 decisions 与 migration plan 中维护 owner/依赖/保留边界。
- [ ] 1.3 维护 `design.md#observable-behavior-matrix` 的 B1–B9；把 commands、flags、help/value names、defaults、source priority、stdout/stderr、exit code、command-shape、selected candidate、stage isolation、output context 和 hyphen-leading spelling 的证明目标分配到 2.5、3.5 和 4.5。

## 2. Typed-field extensions 与 Clap projection

- [ ] 2.1 为 typed-fields 增加 immutable type-indexed consumer extension API：`FieldDefBuilder::extension<E>`、`FieldDef::extension<E>`、`Send + Sync + 'static` payload、shared immutable clone、build/clone/type-erasure/aggregation preservation，以及同 type duplicate 与 missing behavior；不增加 string key、replace/set、mutation 或 `FieldDefSet` retrieval API，底层不依赖或解释 Docnav/Clap 类型。
- [ ] 2.2 新增不依赖 Clap 的 `docnav-field-authoring` shared crate，提供 `DocnavCliPresentation`、`FieldDefBuilder` 项目扩展和 framework-neutral `DocnavFieldProjection`；adapter contracts 与 navigation 通过该 crate 在声明处添加 help、value name、display order 与 Boolean switch/token-map encoding，projection 同时读取 canonical identity、CLI locator、value kind、constraints 和 default。
- [ ] 2.3 让 `cli-config-resolution-clap` 拥有 consumer-neutral `ClapFieldSpec` input，并由 Docnav core 提供 `DocnavFieldProjection -> ClapFieldSpec` 的一对一 bridge；覆盖 string、integer、finite number、Boolean、repeated string array 和 repeated `key=value` object，`ValueKind::Json` 继续拒绝；accepted/default 仅用于 help，enum/range/pattern/required/default 等 canonical semantics 不装入 Clap value validation。
- [ ] 2.4 从 matching projection 和 successful `ArgMatches` 构造 normalized CLI `Source`；成功 decode 生成 typed candidate，失败生成保留 raw value/reason 的 invalid candidate，extension/projection mismatch、match storage mismatch、source construction failure 和 declaration conflict 返回 structural error。
- [ ] 2.5 添加 typed-fields/field-authoring/core-bridge/companion/resolver contract tests，覆盖 immutable extension preservation、same-type duplicate、missing retrieval、CLI locator 与 extension 双向匹配、config-only field 无 extension、bridge exact mapping、Boolean encodings、decode/semantic invalid candidate facts、unselected/overridden invalid non-blocking、selected/merge contributor invalid blocking 及 structural failures；运行子仓库 fmt、clippy、all-target tests、doc tests 和 example。
- [ ] 2.6 将 `cli-config-resolution-clap` 的 projection validation、command augmentation 和 candidate extraction 拆成独立 owner 模块，保持 public contract；同步拆分 touched tests，并删除 `scripts/quality/accepted-warnings.ts` 中被本次 CLI parsing refactor 触发的 acceptance。

## 3. Navigation registry projection 与 selected field set

- [ ] 3.1 使用 Docnav field builder 扩展声明 routing/common fields 的 CLI metadata，并迁移 `adapter`、`page`、`limit`、`output` 和 `pagination.enabled`；fixed `path`/`ref`/`query` positionals、core-owned invocation logging config 与 config-only outline selectors 保持各自 owner/stage 边界。
- [ ] 3.2 让 `AdapterOptionSpec` builder 复用同一项目 field extension，并迁移 built-in adapter native options；operation applicability、constraints/defaults、config paths 和 handler binding 继续由原 declaration 拥有。
- [ ] 3.3 按 operation 聚合 applicable common fields 和 registry adapter native fields，生成 registry projection及 field owner/operation correspondence；确定性拒绝同一 operation 内 duplicate locator 与 projected-to-static conflict。
- [ ] 3.4 Adapter selection 前解析 routing-required fields；selection 后从 common 与 selected adapter/current-operation declarations 重组 `FieldDefSet`，仅把 set 中 identities 对应的 typed/invalid candidates 交给 resolver，并在 selected-set 边界丢弃其它 candidates；command-shape failures 保持为 selection 前 structural failure。
- [ ] 3.5 为 normal document execution 建立 logging、routing、outline policy 和 selected-operation projection：每个 projection 只把 selected field paths 及其必要结构祖先作为正向白名单，只允许其中 facts 产生 candidate、trace、diagnostic 或 request effect；底层顺便计算的 projection 外 facts 必须丢弃。保持 source-level config loading failure 与 registry-wide `config inspect`，并添加 B3–B6 navigation/config/core tests。

## 4. Core authoritative Clap 与 output boundary

- [ ] 4.1 构造完整 root/subcommand Clap tree：core 保留 topology、fixed positionals 和 core-owned static arguments，document subcommands 接入 routing/operation registry projection，并通过唯一 mechanical bridge 把 `DocnavFieldProjection` 转为 companion-owned `ClapFieldSpec`。
- [ ] 4.2 Successful structural parse 后取得 normalized candidates；先校验 explicit `output`，再交付 routing facts。Adapter selection 后的 blocking field validation 只来自 navigation 重组的 selected `FieldDefSet`。
- [ ] 4.3 将 normalized candidates 和 owner/operation correspondence 交给 navigation；删除 raw native strings 作为业务语义输入的路径。
- [ ] 4.4 删除 raw argv output hint：command-shape failure、duplicate output 和 invalid output 使用 PlainText；valid explicit output 通过 canonical validation 后影响所有 later failure，config/default output 仅在 normal navigation resolution 成功后成为 context，并同步 output contract。
- [ ] 4.5 覆盖 B1、B2、B7–B9，以及 root、document operations、management commands、generated help、common fields、adapter native fields、non-selected candidate discard 和 `--flag=<hyphen-leading-value>` 的 CLI/process tests。

## 5. 单路径切换与实现证据

- [ ] 5.1 删除 `docnav-cli-args`、native option catalog、`cli_arg_id()` 推测、string-valued native bridge、post-match arbitrary JSON decoder、authored presentation sidecar、raw output scanner 和 runtime fallback。
- [ ] 5.2 更新 testing/case materials，并审计 schemas、examples、fixtures 和 release artifacts；这些当前实现证据在代码切换后再刷新，且只反映已声明的 CLI failure channel/spelling 与 field processing scope，不改变 document payload、ref 或 adapter result shape。

## 6. 验收

- [ ] 6.1 运行子仓库独立验证，以及主仓库 fmt、targeted Rust tests、case consistency、CLI/process smoke 和 clippy `-D warnings`。
- [ ] 6.2 运行 `bun run verify:docnav-workspace` 和 `openspec validate derive-cli-from-field-definitions --type change --strict --no-interactive`，要求 0 failed。
- [ ] 6.3 按 5.1 的删除清单搜索 legacy 残留，确认 selected-set filtering 未引入 usage-accounting state、stage boundary 不暴露 projection 外 diagnostics、consumer extension 未演化为 replace/mutation/lifecycle framework、touched quality acceptance 已移除，并检查局部 diff 仅包含目标范围。
