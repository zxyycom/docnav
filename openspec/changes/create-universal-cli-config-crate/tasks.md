本 tasks 清单是 `create-universal-cli-config-crate` 的未审核实现计划：先完成 artifact 审计，再抽象通用 crate 边界，随后接入 Docnav 兼容层并验证子仓库化条件；当前文档只存在于 `openspec/changes/create-universal-cli-config-crate/`，不影响现有其它文档或主规范。

## 1. 实现前阻塞审计

- [ ] 1.1 阻塞级审计：在执行任何实现任务前，审计 `proposal.md`、`design.md`、`specs/cli-config-resolution/spec.md` 和本 `tasks.md` 是否围绕“从 Docnav typed-fields 和 parameter-resolution 抽象出可子仓库化复用的 Rust CLI/config resolution crate”这一核心句展开。
- [ ] 1.2 确认 capability ID `cli-config-resolution` 符合长期 owner 命名规则，并确认未误把 change 名称、实现阶段或一次性迁移名作为 capability。
- [ ] 1.3 确认当前 change 只包含 `openspec/changes/create-universal-cli-config-crate/` 下的未审核临时 artifacts，没有修改主规范、docs、schemas、examples 或实现代码。
- [ ] 1.4 确认 `design.md` 的 `## Open Questions` 没有未回答问题；如审计发现 crate/package 名、子仓库化方式或发布节奏会改变 public contract，先更新 design decision 后再进入实现。
- [ ] 1.5 在 1.1-1.4 全部完成前，不得执行 2.x 及后续任何实现任务。

## 2. Crate 边界与现有模型拆分

- [ ] 2.1 读取 `docs/navigation.md` 指向的相关主规范、`docs/coding-style.md`、`docs/testing.md`、`docs/testing/case-maintenance.md`，确认本实现的 owner、验证范围和 Docnav 兼容边界。
- [ ] 2.2 梳理 `crates/shared/typed-fields`、`crates/shared/typed-fields-macros`、`crates/shared/parameter-resolution`、`crates/shared/cli-args` 和 Docnav CLI/native option 调用链，标注可复用核心、Docnav 专属 wrapper 和需要暂缓迁移的行为。
- [ ] 2.3 在 workspace 内建立通用 crate 边界，核心 crate 不依赖 Docnav protocol、adapter contracts、navigation、output 或 Markdown adapter crate。
- [ ] 2.4 保留现有 Docnav resolver 路径或兼容 wrapper，确保迁移期间可对比新旧解析行为并可回退。

## 3. 通用字段契约与 projection

- [ ] 3.1 定义通用 field contract public API，覆盖 stable identity、value kind、constraints、default metadata、projection metadata 和 validation failure facts。
- [ ] 3.2 将现有 processing/projection 概念调整为可表达 CLI、env、config、default 和 custom source 的 projection metadata。
- [ ] 3.3 增加 field set 构建校验，覆盖 duplicate identity、duplicate projection locator、incompatible projection path 和 invalid field declaration。
- [ ] 3.4 为 builder API 增加单元测试，证明字段契约不拥有应用 command、adapter、protocol 或 diagnostic code 语义。
- [ ] 3.5 评估 derive macro 是否进入本阶段；若进入，先让 macro 生成与 builder API 等价的显式 metadata，并新增 compile-fail/compile-pass 测试。

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

- [ ] 6.1 增加 `clap` adapter 或 feature，支持从 field projection 生成/读取 CLI args，同时保持 core crate 在未启用 feature 时不依赖 `clap`。
- [ ] 6.2 增加 JSON/TOML 或 serde-compatible config adapter，支持 structured config source 到 source candidates 的映射。
- [ ] 6.3 增加 explain/debug API，从 provenance trace 输出稳定、可测试的来源解释。
- [ ] 6.4 增加 examples，展示 CLI + env + config + default 的 resolution，以及 list/map merge 和 conflict diagnostics。
- [ ] 6.5 验证 feature flags，确保 core-only build、clap-enabled build、config-enabled build 和 all-features build 都能独立通过。

## 7. Docnav 兼容迁移

- [ ] 7.1 将 Docnav explicit input、project config、user config 和 built-in defaults 映射到通用 source model，保持现有优先级语义。
- [ ] 7.2 将 selected adapter native option declarations 映射到通用 field/projection model，保持 adapter semantics、operation applicability 和 handler binding 在 Docnav owner 内。
- [ ] 7.3 让 navigation input resolution 通过兼容 wrapper 使用新 resolver，保持 protocol request construction、diagnostics projection 和 output behavior 不变。
- [ ] 7.4 添加新旧 resolver 对比测试，覆盖 common navigation fields、outline mode config、adapter native options、unknown config keys 和 invalid typed values。
- [ ] 7.5 移除或标注旧 fixed source resolver 的 fallback；若暂留 fallback，写明原因、约束范围和移除条件。

## 8. 子仓库化与验证

- [ ] 8.1 确认最终 crate/package 名、license、README、feature matrix、examples 和 public API stability notes。
- [ ] 8.2 选择子仓库化方式：独立 repo、Git submodule 或 Git subtree；记录选择理由、迁移影响和回滚路径。
- [ ] 8.3 验证核心 crate 不依赖 Docnav runtime crates，并能作为独立 package 构建和测试。
- [ ] 8.4 运行范围匹配的 Rust tests、macro tests、Docnav parser/config/native option tests。
- [ ] 8.5 若改动跨 Rust 行为、schema/example、CLI/API surface 或多个包边界，运行 `bun run verify:docnav-workspace`。
- [ ] 8.6 用局部 diff 审计只改动目标范围，并确认没有将 Docnav adapter/protocol/output 专属语义泄漏进通用 core。
- [ ] 8.7 更新本 change 的验证记录，确认 `openspec validate create-universal-cli-config-crate --type change --strict --no-interactive` 通过。
