本 tasks 清单是 `create-universal-cli-config-crate` 的 change-local 实现计划：先完成 artifact 审计，再抽象通用 crate 边界，随后 hard cutover Docnav 调用链并验证子仓库化条件；当前文档只存在于 `openspec/changes/create-universal-cli-config-crate/`，不影响主规范及现有其它文档。

## 1. 实现前阻塞审计

- [x] 1.1 阻塞级审计：在执行任何实现任务前，审计 `proposal.md`、`design.md`、`specs/cli-config-resolution/spec.md` 和本 `tasks.md` 是否围绕“从 Docnav typed-fields 和 parameter-resolution 抽象出可子仓库化复用的 Rust CLI/config resolution crate”这一核心句展开。
- [x] 1.2 确认 capability ID `cli-config-resolution` 符合长期 owner 命名规则，并确认未误把 change 名称、实现阶段、一次性迁移名作为 capability。
- [x] 1.3 确认当前 change 只包含 `openspec/changes/create-universal-cli-config-crate/` 下的 change-local artifacts，没有修改主规范、docs、schemas、examples、实现代码。
- [x] 1.4 确认 `design.md` 的 `## Decision Triggers` 没有未回答问题；Docnav hard cutover、crate/package 名、子仓库化路径、发布节奏、derive macro 范围和 framework integration 形态已收敛为默认执行路径。
- [x] 1.5 在 1.1-1.4 全部完成前，不得执行 2.x 及后续任何实现任务。
- [x] 1.6 记录 2026-07-09 prompt-optimize artifact 审阅结论：Docnav 集成采用 hard cutover；工作区实现使用 `cli-config-resolution`，外部 package 名默认沿用 `cli-config-resolution`，子仓库化默认迁移到独立 repository。

## 2. Crate 边界与现有模型拆分

- [ ] 2.1 读取 `docs/navigation.md` 指向的相关主规范、`docs/coding-style.md`、`docs/testing.md`、`docs/testing/case-maintenance.md`，确认本实现的 owner、验证范围和 Docnav hard cutover 边界。
- [ ] 2.2 梳理 `crates/shared/typed-fields`、`crates/shared/typed-fields-macros`、`crates/shared/parameter-resolution`、`crates/shared/cli-args` 和 Docnav CLI/native option 调用链，标注可复用核心、Docnav 专属映射层和必须在 hard cutover 中替换的行为。
- [ ] 2.3 在 workspace 内建立通用 crate 边界，核心 crate 不依赖 Docnav protocol、adapter contracts、navigation、output、Markdown adapter crate。
- [ ] 2.4 建立 hard cutover 验收门：旧 resolver 只作为切换前测试基线保留；本 change 完成时不得保留旧 resolver 运行路径、runtime feature flag 和 fallback 开关。

## 3. 通用字段契约与 projection

- [ ] 3.1 定义通用 field contract public API，覆盖 stable identity、value kind、constraints、default metadata、projection metadata 和 validation failure facts。
- [ ] 3.2 将现有 processing/projection 概念调整为可表达 CLI、env、config、default 和 custom source 的 projection metadata。
- [ ] 3.3 增加 field set 构建校验，覆盖 duplicate identity、duplicate projection locator、incompatible projection path 和 invalid field declaration。
- [ ] 3.4 为 builder API 增加单元测试，证明字段契约不拥有应用 command、adapter、protocol、diagnostic code 语义。
- [ ] 3.5 记录 derive macro 不进入首批实现；确认 builder API 暴露的 metadata 足够支撑后续独立 macro change。

## 4. Source model 与 extraction

- [ ] 4.1 定义 `SourceId`、`SourceKind`、`SourceSpec`、`SourceLocator`、`SourceCandidate` 和 source load/parse/explicitness 状态。
- [ ] 4.2 将 fixed `DirectInput / ProjectConfig / UserConfig / Default` source slots 泛化为 ordered source collection。
- [ ] 4.3 实现 CLI flag source candidate 抽取的核心接口，不把 `clap` 绑定进 core。
- [ ] 4.4 实现 env source candidate 抽取接口，保留 env name locator 和缺失/空值/非法值状态。
- [ ] 4.5 实现 config document path source candidate 抽取接口，支持 JSON-like structured value 与 source path locator。
- [ ] 4.6 实现 static/dynamic default source candidate 抽取，确保 default 是 fallback source 而不是普通显式 source。
- [ ] 4.7 为每类 source 增加单元测试，覆盖 missing、present valid、present invalid、explicit absent 和 locator preservation。

## 5. Resolution、merge strategy 与 trace

- [ ] 5.1 实现 ordered source resolution，支持 deterministic priority、applicability filtering 和 selected/overridden candidate 记录。
- [ ] 5.2 实现 field-level merge strategy，至少覆盖 scalar replace、list append、list replace、map merge、map replace 和 deny-conflict。
- [ ] 5.3 实现 required/optional/default resolution 规则，区分 absent、null、false、empty list、empty map 和 invalid value。
- [ ] 5.4 实现 provenance trace，记录 selected source、overridden candidates、merge contributors、default fallback、invalid candidate 和 missing required facts。
- [ ] 5.5 实现 diagnostics model，确保每个 failure 包含 field identity、source id、source locator、received kind 和 constraint/merge reason。
- [ ] 5.6 实现 typed materialization API，成功时返回应用可消费的 typed values/struct input，失败时阻止返回部分无效结果。
- [ ] 5.7 为 resolution 和 merge 增加 deterministic tests，覆盖不同 source order、相同 priority 冲突、list/map merge、default fallback 和 invalid candidate。

## 6. Framework adapters 与 convenience API

- [ ] 6.1 增加 `cli-config-resolution-clap` companion crate，支持从 field projection 生成/读取 CLI args，同时保持 core crate 不依赖 `clap`。
- [ ] 6.2 增加 `cli-config-resolution-serde` companion crate，支持 serde-compatible structured config source 到 source candidates 的映射。
- [ ] 6.3 增加 explain/debug API，从 provenance trace 输出稳定、可测试的来源解释。
- [ ] 6.4 增加 examples，展示 CLI + env + config + default 的 resolution，以及 list/map merge 和 conflict diagnostics。
- [ ] 6.5 验证 package 边界，确保 core build、clap companion build、serde companion build 和 workspace build 都能独立通过。

## 7. Docnav hard cutover

- [ ] 7.1 将 Docnav explicit input、project config、user config 和 built-in defaults 映射到通用 source model，保持现有优先级语义。
- [ ] 7.2 将 selected adapter native option declarations 映射到通用 field/projection model，保持 adapter semantics、operation applicability 和 handler binding 在 Docnav owner 内。
- [ ] 7.3 将 navigation input resolution 调用链直接切到新 resolver，保持 protocol request construction、diagnostics projection 和 output behavior 不变。
- [ ] 7.4 添加 hard cutover 等价测试，覆盖 common navigation fields、outline mode config、adapter native options、unknown config keys 和 invalid typed values。
- [ ] 7.5 删除旧 fixed source resolver 的运行时路径、runtime feature flag 和 fallback 开关；验证失败时修复新 resolver，不把双路径作为完成状态。

## 8. 子仓库化与验证

- [ ] 8.1 确认外部 package 名 `cli-config-resolution` 的可用性、license、README、package matrix、examples 和 public API stability notes；包名不可用时主动向用户确认新名称。
- [ ] 8.2 准备独立 repository 迁移记录：repo 边界、迁移影响和回滚路径；仓库策略冲突时主动向用户确认。
- [ ] 8.3 验证核心 crate 不依赖 Docnav runtime crates，并能作为独立 package 构建和测试。
- [ ] 8.4 运行范围匹配的 Rust tests、macro tests、Docnav parser/config/native option tests。
- [ ] 8.5 若改动跨 Rust 行为、schema/example、CLI/API surface、多个包边界任一范围，运行 `bun run verify:docnav-workspace`。
- [ ] 8.6 用局部 diff 审计只改动目标范围，并确认没有将 Docnav adapter/protocol/output 专属语义泄漏进通用 core。
- [ ] 8.7 更新本 change 的验证记录，确认 `openspec validate create-universal-cli-config-crate --type change --strict --no-interactive` 通过。
