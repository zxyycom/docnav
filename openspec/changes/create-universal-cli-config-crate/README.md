# create-universal-cli-config-crate

状态：2026-07-10 重新打开。第一轮 46/46 实现和验证证据保留在下文，但不再代表当前契约的最终验收。

当前主承诺是：直接复用 `docnav-typed-fields::FieldDef` / `FieldDefSet` 作为 canonical 标准参数对象，由每个 `FieldDef` 直接声明默认 `Replace` 的 `MergeStrategy`，显式声明 CLI/env/config extraction strategies，并由 `cli-config-resolution` 统一执行来源优先级、merge、校验协调和 provenance。`Parameter` / `ParameterSet` 若出现，只能是 canonical 类型的 re-export、薄别名或 convenience API。

正常使用路径必须只声明一次 canonical field set：clap/env/config extractors 根据 field processing metadata 读取来源，resolver 读取 field 自身的 merge strategy 并按固定顺序执行。Priority 越大越高，同 priority 后注册 source 获胜；`Append` / `MapMerge` 从低到高应用，同级按注册顺序，map 后值覆盖同名 key。Selected/contributing candidate 解码失败会阻断，被 `Replace` 覆盖的非法 candidate 只进入 trace，最终值必须再次通过 canonical validation。Clap 未注册 flag 由 clap 原生拒绝；env/config 未声明项默认静默忽略，不增加通用 `UnknownPolicy` 或 unused-key 扫描。

独立子仓库的物理成果是一个可独立 checkout、build 和 test 的 Cargo workspace，可以包含 typed-fields、typed-fields macros、resolution core、clap companion 和 structured-config companion。`cli-config-resolution` 是主要消费者入口。Docnav 仍拥有 command、config layout、adapter、operation、protocol、diagnostic code 和 output 语义；crates.io 发布仍属于单独审批边界。

## Reopened acceptance target

- `FieldContract` / `FieldSet` 及平行 value/constraint/default/validation 模型不再是 public canonical path。
- `FieldDef` 直接拥有 `MergeStrategy::{Replace, Append, MapMerge, DenyConflict}` metadata，默认 `Replace`；不存在 resolution-owned merge policy table 或单独 `ListReplace` / `MapReplace` public variants。
- `FieldDefSet` 可直接驱动 CLI/env/config extraction、固定 priority/tie/merge ordering、canonical validation 和 typed materialization。
- Selected/contributing decode failure、overridden-invalid trace-only 和 final merged-value validation 都有可执行测试证明。
- Docnav runtime 不再构造 `generic_field_set` 或手工复制 canonical field metadata。
- 独立 Cargo workspace 在独立 checkout 中通过 package tests、doc-tests 和 runnable example，并由 Docnav 通过 revision/pin 或等价 dependency source 消费。
- `openspec instructions apply --change create-universal-cli-config-crate --json` 只有在 9-14 节任务全部完成后才能重新返回 `all_done`。

## Slice A fresh evidence (2026-07-10)

- `docnav-typed-fields`：45 个 unit tests、7 个 integration tests 和 1 个 trybuild test 通过。
- `docnav-typed-fields-macros`：tests 与 doc-tests 通过。
- 受影响范围的 clippy 与 fmt 检查通过。
- 独立 review 结论为 `Approve`；初次审查提出的 2 个 High 均已修复并通过复审。
- 本 slice 只完成并记账 9.3 与 10.1；9.2、9.4、9.5 及其它任务仍保持未完成。

## Slice B fresh evidence (2026-07-10)

- Core facade 现在直接依赖并选择性 re-export canonical `FieldDef` / `FieldDefSet`、builder 所需类型及 `Parameter` / `ParameterSet` aliases；不 re-export `FieldDefs`，使用 derive 的消费者仍需直接依赖 typed-fields。
- 重复 field/value/merge model 已删除；source/candidate API、env iterator extraction、static default fallback、priority/later-wins、四项 merge strategy、invalid-candidate timing、final canonical validation、all-or-nothing materialization 和必要 provenance 均由 core 实现及 9 个 integration tests 覆盖。
- 初次独立 review 报告 1 个 High 和 2 个 Medium，覆盖 facade completeness、canonical merge/validation timing 与 public error behavior；三项均以回归测试先行修复，随后两轮独立复审结论均为 `Approve`。
- Core integration tests（9 个）、doc-tests、clippy、fmt 和局部 diff check 通过。
- 本 slice 完成并记账 9.2、9.4、10.3、10.5、11.1-11.6。
- 9.5 保持未完成：当前证据尚未用一个单一 public test 同时贯穿 `FieldDefSet` extraction→resolution→validation→typed materialization，并证明 scalar/list/map 的默认 `Replace`。
- Companion crates、Docnav hard cutover、独立子仓库和 full workspace verification 未在本 slice 验证，不构成相应任务的完成证据。

## Slice C1 fresh evidence (2026-07-10)

- Structured-config companion 直接消费 canonical `FieldDefSet` 的 `ConfigPath` processing metadata，只读取已声明路径；未声明 config keys 静默忽略。
- Missing path 和 non-object traversal 不产生 candidate；已存在的 `null`、`false`、空数组与空对象保持原值并进入统一 core `Source`。
- 旧 `Value`、`FieldSet` 和 `SourceExtractor` 平行模型已删除。
- 4 个 tests、doc-tests、clippy、fmt 和局部 diff check 通过；独立 review 结论为 `Approve`。
- 本 slice 只完成并记账 10.4，当前进度为 60/75；9.5、10.2、10.6 及后续任务保持未完成。Full workspace verification 未运行，不构成相应完成证据。

## Slice C2 fresh evidence (2026-07-10)

- Clap companion 的 canonical `augment_command` / `extract_cli` 直接消费 `CliFlag` processing metadata，并输出统一 core candidates；旧平行 field/value/source 模型已删除。
- 未知 flag 由 clap 原生拒绝；未提供的已声明 flag 不产生 candidate；非法值保留 raw value 和 reason，负数可作为 flag value 读取。
- Implicit argument groups 以及与直接 subcommand flags/aliases 的 locator collision 都通过 `Result` 确定报错，不触发 panic。
- 8 个 lib tests、doc-tests、clippy、fmt 和局部 diff check 通过；独立 review 结论为 `Approve`；`WB-PARAM-CLAP-001` case ledger row 已对齐。
- 本 slice 只完成并记账 10.2，当前进度为 61/75；9.5、10.6 及后续任务保持未完成。旧 runnable example 尚未 cut over，full workspace verification 未运行，不构成相应完成证据。

## Slice C3 fresh evidence (2026-07-10)

- Runnable example 以一个 `FieldDefs` declaration 构建同一个 canonical `FieldDefSet`，并让它完整贯穿 CLI/env/config/static default extraction → `Resolver` → final canonical validation → `FieldValueMap` → typed `Parameters`；没有第二套参数声明。
- 未显式声明 merge strategy 的 scalar、list 和 map 都由竞争来源的最终选值证明默认 `Replace`；显式 `Append`、source priority，以及 selected、contributors、default fallback provenance 均有断言。
- 同一 pipeline 的 3 个有效场景成功 materialize；4 个约束失败场景均产生 `FinalValidation`，且 typed materialization 失败，不返回部分有效参数对象。
- Example tests（2 个）、runnable example、两个 companion package 的 tests、受影响 package 的 all-target clippy、fmt 和局部 diff check 均通过；独立 review 结论为 `Approve`。
- 本 slice 只完成并记账 9.5 与 10.6，当前进度为 63/75；12.x、13.x、14.x 保持未完成。Full Docnav tests 和 `bun run verify:docnav-workspace` 未运行，不构成 hard cutover、独立子仓库或最终验收证据。

## Slice D2 fresh evidence (2026-07-10)

- D1 已先冻结 source locator 语义；D2 让 Docnav existing `FieldDefSet` 直接进入统一 `Source` / `Resolver` path，不再构造或转换第二套字段模型。
- Source priority 保持确定：common/native explicit 为 400、project config 为 300、user config 为 200，static default 从 canonical metadata 自动接入 fallback。
- Docnav owner 继续负责 strict config、JSON native-option 解析和 output projection；optional native option 的 non-JSON 值继续兼容地投影为 `null`。
- 残留审计确认旧 `generic_field_set`、`FieldContract`、`SourceCollection`、`ParameterResolution`、compat wrapper 和 fallback path 均为零残留。
- Fresh verification：navigation 47 tests、config-inspect 6 tests、consumer compile、clippy 和 fmt 均通过。
- Review/remediation：三轮 bounded challenge 均未发现 valid issue，formal review 结论为 `Approve`；没有遗留的 review finding 需要在本次记账前继续 remediation。
- 本 slice 只完成并记账 12.1 与 12.2，当前进度为 65/75。D3 golden/smoke/workspace verification 尚未完成，因此 12.3、13.x 和 14.x 保持未完成。

## Current verification and repository status (2026-07-11)

- Hard-cutover baseline（D3）：golden、case ledger、owner docs、48 个 smoke cases 和当时的 workspace gate 均已完成；D3 formal review 在唯一 Low 关闭后为 `Approve`。
- E1 repository boundary：`subrepos/cli-config-resolution/` 已形成包含 `docnav-typed-fields`、`docnav-typed-fields-macros`、resolution core、clap companion 和 structured-config companion 的五包 nested Cargo workspace。Docnav root workspace 排除这些 nested members，并通过 path dependencies 消费；root/nested 两份 lockfile 均存在，source uniqueness audit 只发现一份 owner source。`cli-config-resolution` 是 re-export canonical 参数类型的主要入口，nested packages 不依赖 Docnav protocol、adapter、navigation、output 或 Markdown crates；E1 review 为 `Approve`。
- E2 package readiness：workspace/package README 与 metadata 已把 canonical single-declaration 写成 normal path。Nested metadata、build、clippy、76 tests、doc-tests、rustdoc 和 runnable example 均通过，E2 review 为 `Approve`。Canonical public repository 已确认为 `https://github.com/zxyycom/cli-config-resolution`，五个 package 统一继承该 repository metadata；用户明确将 license selection 延后到 release decision，因此当前 metadata 为 `license = null`，且不增加 `LICENSE` 文件。
- E3 automation：required/full verifier 均指向 nested manifest；quality scan 覆盖 nested production/tests/examples/fixtures/benches 并排除 nested `target`；case validator 扫描精确 nested root，结果为 101 documented / 101 source markers。E3 review 为 `Approve`；fresh full workspace verifier 共 19 checks，18 passed、1 warning、0 failed。
- Fresh raw quality snapshot 为 `all=24`、`changed=18`、`regressions=18`，仍是非阻断观测。新增的 3 条 regression 是移动后的 path-key 断裂：`resolution.rs` 515 vs old 647、clap `lib.rs` 414 vs old 434、`source.rs` 304 vs old 487；`all` 增加的 1 条是拆出的 clap `tests.rs` 334。该 raw 结果不支持把 18 条 regression 全部声称为真实增长；没有新增 `acceptedReason`，既有 owner 与 follow-up 边界继续如下：

| Owner | 当前保留原因 | 后续处理边界 |
| --- | --- | --- |
| `docnav-typed-fields` | locator uniqueness 与 set build validation 当前共同维护同一组 canonical build invariants。 | 新增 locator 或 build rule 时抽出 `ProcessingLocationRegistry`。 |
| `cli-config-resolution-clap` | 当前 candidate extraction 与 locator conflict checks 共同覆盖既定 value kinds，已有 tests 固定行为。 | 新增 value kind 时拆出 candidate/conflict helpers。 |
| `cli-config-resolution` resolver | 当前 selection、四项 merge strategy 与 static-default fallback 共享同一套 source ordering facts。 | 新增 merge strategy 或 default behavior 时拆出 selection 模块。 |
| Docnav `config-inspect` | 当前 projection surface 已由 hard-cutover golden 固定，没有新增独立输出职责。 | 扩展 `config-inspect` 时拆出 projection 模块。 |

- Test/example 超阈值、实现净删除和既有未增长项继续按 D3 分类保留；后续只在上表的职责扩张条件触发时拆分，不隐藏 warning。
- Final structure audit 为 `Approve`，完成 14.4：没有第二套 field/value/merge model、旧 resolver 或 Docnav conversion path，也没有把 Docnav-specific semantics 泄漏进 nested workspace。
- Remaining E4：公开仓库 `https://github.com/zxyycom/cli-config-resolution` 已建立，owner 为 `zxyycom`，公开可见性与 GitHub 操作授权已确认。实际 source push、submodule 采用记录、真实 revision pin 和独立 checkout 验证尚未完成，因此 13.1、13.3、13.4 保持未完成。License selection 已明确延后，不在本轮代选；14.2 等待 Git 结果与最终文档审计后统一记账，14.5 继续依赖全部任务完成。
- 本轮完成并记账 13.2、14.1、14.4；当前进度为 70/75。

## First-round implementation record

以下材料记录第一轮实现当时的 package、迁移准备和验证事实，用于追踪已经完成过的工作。它们不是当前 API 收敛与子仓库验收的替代证据。

### 8.1 Release readiness at first acceptance

#### Package name evidence

在 2026-07-10T01:07:29Z，crates.io 官方 API 对以下 package 名均返回 HTTP 404：

- `cli-config-resolution`
- `cli-config-resolution-clap`
- `cli-config-resolution-serde`

该结果只证明检查时没有对应 crate 记录，不构成名称预留或后续可用性承诺。任何外部发布动作前必须重新检查名称。

#### License evidence

第一轮验收时，三个 package manifest 均通过 `license.workspace = true` 继承 workspace 的 MIT SPDX metadata。该选择已由用户 2026-07-11 的决定取代；当前五个 package metadata 均为 `license = null`，license selection 延后到 release decision。

#### Package matrix

| Package | Consumer role | Normal dependencies | Consumer material |
| --- | --- | --- | --- |
| `cli-config-resolution` | Framework-independent field/source contracts、ordered resolution、merge、diagnostics、materialization 和 provenance | 无 | package README 的 default-source 最小示例 |
| `cli-config-resolution-clap` | 从 CLI projection 构造 `clap` arguments，并将 matches 映射为 core candidates | `cli-config-resolution`、`clap` | package README；`examples/resolution_flow.rs` |
| `cli-config-resolution-serde` | 将 `serde_json::Value` 的 config path 映射为 core candidates | `cli-config-resolution`、`serde_json` | package README；由 `resolution_flow` 联合覆盖 |

三个 README 都只描述 package 消费者需要的用途、入口和稳定性边界。`resolution_flow` 是跨 package 的 runnable example，覆盖 CLI、env、JSON config、default、list/map merge、conflict diagnostic 和 provenance explain；运行入口为 `cargo run -p cli-config-resolution-clap --example resolution_flow`。

当前 workspace version 为 `0.1.0`。三个 README 均明确记录 pre-1.0 public API 尚无兼容性保证；本记录不承诺发布日期、发布节奏或后续 API 稳定级别。derive macro 继续留在后续独立 change。

### 8.2 Independent repository migration preparation at first acceptance

#### Status and repository metadata

本节记录第一轮验收时的迁移准备状态：当时尚未执行 repository 创建、代码搬迁或外部发布，三个 package manifest 因而没有 `repository` URL。重新打开后的 13.x 任务已经把独立 Cargo workspace 子仓库纳入当前完成条件；本段只保留历史事实。

#### Repository boundary

迁入独立 repository 的边界：

- 三个 package 的 source、manifest、package README、tests，以及 `cli-config-resolution-clap/examples/resolution_flow.rs`。
- 第一轮计划由目标 repository 的 workspace manifest 承接三个 package 所需的 version、edition、MIT SPDX 和 shared dependency metadata；其中 MIT 选择已由用户 2026-07-11 的决定取代，当前 workspace 不声明 license。
- core 保持不依赖 Docnav protocol、adapter contracts、navigation、output 和 Markdown adapter crates；framework dependency 继续只存在于 companion package。

保留在 Docnav repository 的边界：

- Docnav command/config layout、adapter/operation applicability、protocol/output projection 和 diagnostic code mapping。
- Docnav hard-cutover integration、等价测试、workspace 验证链和本 OpenSpec change 的审计历史。

#### Migration impact

1. Package 和 Rust crate 名保持不变，消费者 import path 不因 repository 迁移而重命名。
2. 独立 repository 必须先提供可解析的 core dependency，companion package 才能脱离当前 workspace path dependency；迁移验证不得依赖 Docnav 本地路径。
3. Docnav 随后把三个 workspace dependency 从本地 path 切换到经批准的外部 source，并更新 lockfile；该 source 的类型和真实 URL 由迁移执行时确认，本记录不预设。
4. 依赖 source 切换不得改变 Docnav 的 hard-cutover runtime behavior。切换后需要重跑 package tests、Docnav config/native-option 等价测试和 workspace verification。
5. crates.io 名称必须在任何发布动作前重新检查；本次 404 证据不能替代执行时检查。

#### Rollback path

1. 在 Docnav dependency source 切换前，迁移可以停止，三个 package 继续作为当前 workspace member 构建和测试。
2. 若独立 repository 验证失败，修复在独立边界完成；需要撤回迁移时，恢复当前 package directories 和 path dependency metadata，不引入临时兼容 wrapper。
3. 若 Docnav 已切换 dependency source但验证失败，回滚 dependency metadata 和 lockfile 到已验证的 workspace path source，并重跑 hard-cutover 等价测试。
4. repository 迁移的回滚只回退 package location 和 dependency source，不恢复旧 fixed source resolver、runtime feature flag 或 fallback path。
5. 外部发布属于单独审批边界；本记录不创建发布 artifact，也不把外部 artifact 的处置作为可由代码 revert 完成的回滚步骤。

#### Migration gates

- 独立 repository owner、canonical URL 和 repository policy 已真实确认。
- 三个 package 在独立 checkout 中通过 metadata、package listing、tests 和 example verification。
- package 名在发布动作前重新检查，version 和发布顺序另行批准。
- Docnav dependency source 切换后通过 hard-cutover 等价测试和 workspace verification。

### First-round verification evidence

#### Acceptance recorded at 2026-07-10T13:18:57+08:00

以下结果在第一轮验收时是 fresh evidence；change 重新打开后，它们保留为历史基线，不得被复用为 9-14 节的最终验证结果。

- 格式与 lint：`cargo fmt --all -- --check` 通过；三个新 package 的 `cargo clippy --all-targets -- -D warnings` 通过。
- Package tests：core 18 passed；clap companion 5 passed；serde companion 4 passed；各自 doc-tests 均 0 failed。
- Docnav tests：`docnav-typed-fields` 45 unit tests 加 1 compile test passed；`docnav-typed-fields-macros` package 存在且 unit/doc-tests 0 failed；`docnav-navigation --lib` 42 passed；`docnav --lib` 92 passed。
- Case ledger：`bun run validate:docs -- cases` 通过，101 implemented、1 planned、101 unique source markers。
- Package boundary：`cargo metadata --no-deps --format-version 1` 显示三个新 package 均存在并已被各自测试命令独立选择，旧 `parameter-resolution` package 缺席；core 的 dependency 列表为空。`cargo tree` 进一步显示 core 仅包含自身，clap/serde companion 仅引入各自 framework dependency 与 core。
- Workspace gate：`bun run verify:docnav-workspace` 退出码为 0，14 项检查中 13 passed、1 warning、0 failed。唯一 warning 是非阻断 `quality full check`：24 条未接受原因记录（20 changed、20 regressions），细分为 10 条 file code-lines、9 条 function code-density、4 条 cyclomatic complexity、1 条 parameter count；完整报告位于 `artifacts/docnav-quality/report.md`。同次 gate 的 clippy、tests、format、docs、OpenSpec、smoke 和 duplication checks 均通过。
- Scope audit：`git diff --check` 退出码为 0，仅输出 `Cargo.lock` 的 LF-to-CRLF working-copy advisory。status/diff 范围限定在 workspace manifests/lockfile、三个新 package、Docnav/navigation hard cutover、owner docs/case ledger 和本 change artifacts；core source/manifest 未出现 Docnav、adapter、protocol、output 或 Markdown 专属语义。旧 resolver package、运行时 import、feature/fallback path 均缺席；原目录下文件数为 0。
- OpenSpec：记账前与记账后的 `openspec validate create-universal-cli-config-crate --type change --strict --no-interactive` 均退出 0；记账后 `openspec instructions apply --change create-universal-cli-config-crate --json` 返回 `state=all_done`、46/46 complete、0 remaining。
- 当时的发布边界：三个 crates.io package 名的可用性只是 2026-07-10T01:07:29Z 的时间点证据，不构成预留；当时真实独立 repository 尚未创建。重新打开后的子仓库创建结果和 fresh verification 应追加到新的最终验收记录，不能改写本历史条目。
