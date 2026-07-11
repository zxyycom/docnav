本 tasks 清单是 `create-universal-cli-config-crate` 的 change-local 实现计划。1-8 节保留第一轮 46/46 实现与验收历史；9-14 节是 2026-07-10 用户确认后重新打开的收敛任务，目标是复用 canonical `FieldDef` / `FieldDefSet`、打通显式 CLI/env/config extraction，并建立独立 Cargo workspace 子仓库。当前文档只存在于本 change 目录，不直接修改主规范。

## 1. 实现前阻塞审计

- [x] 1.1 阻塞级审计：在执行任何实现任务前，审计 `proposal.md`、`design.md`、`specs/cli-config-resolution/spec.md` 和本 `tasks.md` 是否围绕“从 Docnav typed-fields 和 parameter-resolution 抽象出可子仓库化复用的 Rust CLI/config resolution crate”这一核心句展开。
- [x] 1.2 确认 capability ID `cli-config-resolution` 符合长期 owner 命名规则，并确认未误把 change 名称、实现阶段、一次性迁移名作为 capability。
- [x] 1.3 确认当前 change 只包含 `openspec/changes/create-universal-cli-config-crate/` 下的 change-local artifacts，没有修改主规范、docs、schemas、examples、实现代码。
- [x] 1.4 确认 `design.md` 的 `## Decision Triggers` 没有未回答问题；Docnav hard cutover、crate/package 名、子仓库化路径、发布节奏、derive macro 范围和 framework integration 形态已收敛为默认执行路径。
- [x] 1.5 在 1.1-1.4 全部完成前，不得执行 2.x 及后续任何实现任务。
- [x] 1.6 记录 2026-07-09 prompt-optimize artifact 审阅结论：Docnav 集成采用 hard cutover；工作区实现使用 `cli-config-resolution`，外部 package 名默认沿用 `cli-config-resolution`，子仓库化默认迁移到独立 repository。

## 2. Crate 边界与现有模型拆分

- [x] 2.1 读取 `docs/navigation.md` 指向的相关主规范、`docs/coding-style.md`、`docs/testing.md`、`docs/testing/case-maintenance.md`，确认本实现的 owner、验证范围和 Docnav hard cutover 边界。
- [x] 2.2 梳理 `crates/shared/typed-fields`、`crates/shared/typed-fields-macros`、`crates/shared/parameter-resolution`、`crates/shared/cli-args` 和 Docnav CLI/native option 调用链，标注可复用核心、Docnav 专属映射层和必须在 hard cutover 中替换的行为。
- [x] 2.3 在 workspace 内建立通用 crate 边界，核心 crate 不依赖 Docnav protocol、adapter contracts、navigation、output、Markdown adapter crate。
- [x] 2.4 建立 hard cutover 验收门：旧 resolver 只作为切换前测试基线保留；本 change 完成时不得保留旧 resolver 运行路径、runtime feature flag 和 fallback 开关。

## 3. 通用字段契约与 projection

- [x] 3.1 定义通用 field contract public API，覆盖 stable identity、value kind、constraints、default metadata、projection metadata 和 validation failure facts。
- [x] 3.2 将现有 processing/projection 概念调整为可表达 CLI、env、config、default 和 custom source 的 projection metadata。
- [x] 3.3 增加 field set 构建校验，覆盖 duplicate identity、duplicate projection locator、incompatible projection path 和 invalid field declaration。
- [x] 3.4 为 builder API 增加单元测试，证明字段契约不拥有应用 command、adapter、protocol、diagnostic code 语义。
- [x] 3.5 记录 derive macro 不进入首批实现；确认 builder API 暴露的 metadata 足够支撑后续独立 macro change。

## 4. Source model 与 extraction

- [x] 4.1 定义 `SourceId`、`SourceKind`、`SourceSpec`、`SourceLocator`、`SourceCandidate` 和 source load/parse/explicitness 状态。
- [x] 4.2 将 fixed `DirectInput / ProjectConfig / UserConfig / Default` source slots 泛化为 ordered source collection。
- [x] 4.3 实现 CLI flag source candidate 抽取的核心接口，不把 `clap` 绑定进 core。
- [x] 4.4 实现 env source candidate 抽取接口，保留 env name locator 和缺失/空值/非法值状态。
- [x] 4.5 实现 config document path source candidate 抽取接口，支持 JSON-like structured value 与 source path locator。
- [x] 4.6 实现 static/dynamic default source candidate 抽取，确保 default 是 fallback source 而不是普通显式 source。
- [x] 4.7 为每类 source 增加单元测试，覆盖 missing、present valid、present invalid、explicit absent 和 locator preservation。

## 5. Resolution、merge strategy 与 trace

- [x] 5.1 实现 ordered source resolution，支持 deterministic priority、applicability filtering 和 selected/overridden candidate 记录。
- [x] 5.2 第一轮历史实现曾覆盖 scalar replace、list append、list replace、map merge、map replace 和 deny-conflict；当前 public surface 已由 9.4 收敛，`ListReplace` / `MapReplace` 不作为保留要求。
- [x] 5.3 实现 required/optional/default resolution 规则，区分 absent、null、false、empty list、empty map 和 invalid value。
- [x] 5.4 实现 provenance trace，记录 selected source、overridden candidates、merge contributors、default fallback、invalid candidate 和 missing required facts。
- [x] 5.5 实现 diagnostics model，确保每个 failure 包含 field identity、source id、source locator、received kind 和 constraint/merge reason。
- [x] 5.6 实现 typed materialization API，成功时返回应用可消费的 typed values/struct input，失败时阻止返回部分无效结果。
- [x] 5.7 为 resolution 和 merge 增加 deterministic tests，覆盖不同 source order、相同 priority 冲突、list/map merge、default fallback 和 invalid candidate。

## 6. Framework adapters 与 convenience API

- [x] 6.1 增加 `cli-config-resolution-clap` companion crate，支持从 field projection 生成/读取 CLI args，同时保持 core crate 不依赖 `clap`。
- [x] 6.2 增加 `cli-config-resolution-serde` companion crate，支持 serde-compatible structured config source 到 source candidates 的映射。
- [x] 6.3 增加 explain/debug API，从 provenance trace 输出稳定、可测试的来源解释。
- [x] 6.4 增加 examples，展示 CLI + env + config + default 的 resolution，以及 list/map merge 和 conflict diagnostics。
- [x] 6.5 验证 package 边界，确保 core build、clap companion build、serde companion build 和 workspace build 都能独立通过。

## 7. Docnav hard cutover

- [x] 7.1 将 Docnav explicit input、project config、user config 和 built-in defaults 映射到通用 source model，保持现有优先级语义。
- [x] 7.2 将 selected adapter native option declarations 映射到通用 field/projection model，保持 adapter semantics、operation applicability 和 handler binding 在 Docnav owner 内。
- [x] 7.3 将 navigation input resolution 调用链直接切到新 resolver，保持 protocol request construction、diagnostics projection 和 output behavior 不变。
- [x] 7.4 添加 hard cutover 等价测试，覆盖 common navigation fields、outline mode config、adapter native options、unknown config keys 和 invalid typed values。
- [x] 7.5 删除旧 fixed source resolver 的运行时路径、runtime feature flag 和 fallback 开关；验证失败时修复新 resolver，不把双路径作为完成状态。

## 8. 子仓库化与验证

- [x] 8.1 确认外部 package 名 `cli-config-resolution` 的可用性、license、README、package matrix、examples 和 public API stability notes；包名不可用时主动向用户确认新名称。
- [x] 8.2 准备独立 repository 迁移记录：repo 边界、迁移影响和回滚路径；仓库策略冲突时主动向用户确认。
- [x] 8.3 验证核心 crate 不依赖 Docnav runtime crates，并能作为独立 package 构建和测试。
- [x] 8.4 运行范围匹配的 Rust tests、macro tests、Docnav parser/config/native option tests。
- [x] 8.5 若改动跨 Rust 行为、schema/example、CLI/API surface、多个包边界任一范围，运行 `bun run verify:docnav-workspace`。
- [x] 8.6 用局部 diff 审计只改动目标范围，并确认没有将 Docnav adapter/protocol/output 专属语义泄漏进通用 core。
- [x] 8.7 更新本 change 的验证记录，确认 `openspec validate create-universal-cli-config-crate --type change --strict --no-interactive` 通过。

## 9. Canonical 参数 API 收敛

- [x] 9.1 将用户确认的目标写入 proposal、design、delta spec、tasks 和 README：`docnav-typed-fields::FieldDef` / `FieldDefSet` 是 canonical 参数模型；此前 46/46 作为历史记录保留，不再代表最终验收。
- [x] 9.2 让 `cli-config-resolution` 直接依赖并 re-export canonical `FieldDef` / `FieldDefSet`，只为 `Parameter` / `ParameterSet` 提供无状态薄别名或 convenience API。
- [x] 9.3 公开 extractor/resolver 必需的 canonical immutable metadata 与 validation entry points，保持类型、约束、默认值、`MergeStrategy`、set 构建校验和 typed materialization 的 owner 在 typed-fields。
- [x] 9.4 将 `MergeStrategy::{Replace, Append, MapMerge, DenyConflict}` 直接加入 canonical `FieldDef` metadata，默认 `Replace`；删除独立 merge declaration、`ListReplace` / `MapReplace` public variants，以及重复的 `FieldContract`、`FieldSet`、value kind、constraints、default 和 validation 实现。
- [x] 9.5 增加 public-API/compile tests，证明消费者只声明一次 `FieldDefSet` 即可完成 extraction、resolution、validation 和 typed materialization，并证明未声明 merge strategy 时 scalar/list/map 均为 `Replace`。Slice C3 已由同一 runnable example/test 的单一 `FieldDefs` declaration 贯穿两组要求。

## 10. 显式 extraction strategies

- [x] 10.1 在现有 `ProcessStrategy` / processing metadata 上提供明确的 CLI flag、env var 和 config path strategy，并验证 duplicate locator 与不合法 locator 在 set build 或 adapter build 阶段确定失败。
- [x] 10.2 整理 `cli-config-resolution-clap`，使其直接消费 canonical CLI strategy、注册或读取 clap arguments 并返回统一 candidates；未知 flag 由 clap 原生拒绝。
- [x] 10.3 提供可注入 key/value iterator 的 env extractor，只查询声明的 env locators；未声明变量静默忽略，缺失变量不产生 effective candidate。
- [x] 10.4 整理 structured-config companion，使其从当前支持的 structured document 中只读取 canonical config-path strategies；未声明 config keys 静默忽略。
- [x] 10.5 从 `FieldDef` static default metadata 自动生成 fallback；dynamic default 继续通过显式 source 接入，不要求消费者手工拼 static-default candidates。
- [x] 10.6 更新一个 runnable example，完整展示单次参数声明、CLI + env + config + default extraction、canonical field merge strategy、确定的 source order、canonical validation、typed materialization 和 provenance。

## 11. Resolution、merge 与 provenance 整理

- [x] 11.1 收敛 source/candidate public API，使 extractor 输出可以直接交给 resolver；删除不参与决策且可由 source/candidate presence 或 diagnostic 表达的平行 load/explicitness/missing 状态。
- [x] 11.2 固定 source ordering：priority 数值越大越高，同 priority 后注册 source 获胜；增加 `Replace`、equal-priority tie 和 custom-source tests。
- [x] 11.3 Resolver 只执行 `FieldDef` 上的 `Replace`、`Append`、`MapMerge`、`DenyConflict`：`Append` / `MapMerge` 按低 priority 到高 priority、同级注册顺序应用，`MapMerge` 后值覆盖同名 key。
- [x] 11.4 复用 canonical metadata 解码 candidates：selected 或 contributing candidate 解码失败即阻断；`Replace` 下被覆盖的非法 candidate 只进入 trace；最终 selected/merged value 必须再次通过 canonical `FieldDef` validation 后才能交给 typed materialization。
- [x] 11.5 收敛 provenance/diagnostics 为 selected、overridden、contributors、default fallback、invalid input 和 missing required 的必要事实，确保被覆盖的非法 candidate 及其 decode failure 可追踪但不成为阻断 diagnostic。
- [x] 11.6 增加 focused tests，覆盖 priority 数值方向、equal-priority later-wins、missing input、selected/contributing decode failure、overridden invalid trace-only、default、`Replace` on scalar/list/map、ordered `Append`、map key overwrite、conflict、final validation 和 materialization failure。

## 12. Docnav canonical hard cutover

- [x] 12.1 删除 `generic_field_set` 和 canonical metadata 到第二套字段模型的转换；Docnav existing `FieldDefSet` 直接进入公共 extractor/resolver path。
- [x] 12.2 通过统一 source API 接入 explicit input、project config、user config、built-in defaults 和 selected adapter native options，同时保留 adapter applicability、handler binding、request construction 与 diagnostic-code mapping 在 Docnav owner。
- [x] 12.3 更新 hard-cutover 等价测试，证明 CLI/config/native-option/public output 行为保持，并确认 runtime path 不含旧 resolver、field-model compatibility wrapper、feature flag 或 fallback。

## 13. 独立 Cargo workspace 子仓库

- [x] 13.1 建立可独立 checkout 的 Git 子仓库/Cargo workspace，包含 canonical typed-fields、typed-fields macros、resolution core、clap companion 和 structured-config companion；workspace root 统一 package metadata 与 shared dependencies。
- [x] 13.2 以 `cli-config-resolution` 作为主要消费者入口并 re-export canonical 参数类型；验证整个子仓库不依赖 Docnav protocol、adapter contracts、navigation、output 或 Markdown adapter crates。
- [x] 13.3 记录并验证 Docnav 的 `.gitmodules` URL、gitlink revision、初始化命令、回滚路径和 lockfile 更新；仓库位置切换不得恢复旧 resolver 或改变参数语义。
- [x] 13.4 在独立 checkout 中运行 metadata、build、tests、doc-tests 和 runnable example，再从 Docnav consumer 运行对应 integration tests。

## 14. 最终验收与记账

- [x] 14.1 运行受影响 packages 的 `cargo fmt --check`、`cargo clippy --all-targets -- -D warnings`、unit/integration/doc tests 和 API compile tests。
- [x] 14.2 更新 package README、workspace README、package matrix、repository metadata 和 example，使正常使用路径只出现 canonical 参数声明；显式记录 license selection 延后到 release decision。
- [x] 14.3 更新测试 case ledger，并运行 `bun run verify:docnav-workspace`；任何非阻断 warning 记录 owner、原因和后续处理边界。
- [x] 14.4 用局部 diff 审计 duplicate field model 与 Docnav conversion path 已移除，且改动没有把 Docnav-specific semantics 泄漏到子仓库。
- [x] 14.5 更新本 README 的 fresh verification evidence，运行 strict OpenSpec validation，并确认 `openspec instructions apply` 返回全部重新打开任务完成后才能归档。
