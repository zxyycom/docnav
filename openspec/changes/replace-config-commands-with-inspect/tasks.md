本 tasks 只在 `openspec/changes/replace-config-commands-with-inspect/` 下形成 change-stage checklist；实现任务执行前必须先完成参数汇总、config inspect scope、adapter-id config path 迁移和数组配置校验边界的审计门禁。

## 文档重心

本 tasks 是推进顺序和退出门禁，不是需求来源。任务项只能引用 `proposal.md`、`design.md` 和 `specs/*/spec.md` 已确定的目标；如果执行中发现任务与契约冲突，先修正 owner artifact，再继续勾选任务。

## 1. 阻塞级审计门禁

- [x] 1.1 审计 proposal、design、specs 和 tasks 是否围绕三个明确目标：单一只读 `docnav config inspect`、复用现有 CLI/input owner-map 与 parameter-resolution 投影、迁移 adapter native option 持久配置路径到 `options.<adapter-id>.<option-key>`。
- [x] 1.2 审计 capability ID 是否只复用 `core-cli`、`navigation-input-resolution`、`typed-fields` 和 `adapter-contract`，且没有把 change name 或宽泛 config umbrella 当作新 capability。
- [x] 1.3 审计本 change 是否明确保留既有 owner 边界：core 拥有 CLI surface，navigation 拥有 source loading、source priority、selected adapter/operation resolution，typed-fields 拥有 field facts/projection helper，adapter 拥有 native option declaration，diagnostics/output 拥有 public projection。
- [x] 1.4 审计 `config inspect` 是否只做 source inspection，报告 selected config sources、load state、source summary、validation diagnostics 和当前输入可解析出的参数事实，不预演 adapter dispatch 或替代 selected-operation validation。
- [x] 1.5 审计 adapter-id path 是否被当作明确配置路径迁移处理，并覆盖 hard-coded path、registry lookup、schema/example/test/docs 同步；不得把它写成 inspect 输出细节或 adapter handler payload change。
- [x] 1.6 审计 `outline.mode_rules[]`、`outline.auto_full_read.thresholds[]` 等非结构化全文读取相关数组配置是否先复用现有 owner-specific validation；typed-fields 扩展只能在 parity 不足时作为最小 subset 进入实现。
- [x] 1.7 审计本 change 是否不改变 raw protocol envelope、readable output wrapper、linked adapter handler payload 或 adapter-owned typed handoff boundary。

## 2. Owner 主规范同步

- [ ] 2.1 更新 `docs/cli.md`，删除 `docnav config get|set|unset|list` 长期 surface，固定单一 source-scoped `docnav config inspect` surface、破坏式迁移边界、当前可解析参数事实展示边界，以及它不是 config editor 或 dispatch preview。
- [ ] 2.2 更新 `docs/navigation-input-resolution.md`，声明参数汇总如何从 owner-provided metadata 产出 CLI/input projection 和 config-source projection，并如何驱动 config source validation 与 selected adapter/operation resolution 的两阶段边界。
- [ ] 2.3 更新 `docs/navigation-input-resolution.md`、`docs/adapter-contract.md` 和 adapter owner 文档，迁移 native option persistent config path 到 `options.<adapter-id>.<option-key>`；明确旧裸 `options.<key>` 按普通 unknown/invalid config path 处理。
- [ ] 2.4 更新 `docs/architecture.md`，把现有 CLI flag owner-map、`ParameterResolutionPipeline`、`FieldDefSet` 和 `AdapterOptionSpec` 关系写成参数汇总边界，而不是新增 parallel aggregation layer。
- [ ] 2.5 更新 `docs/testing.md`、`docs/testing/case-maintenance.md` 或 case ledger 中对应条目，写明 config inspect/read/resolution parity、adapter-id option namespace、非结构化全文读取数组配置校验、旧 config 子命令移除和 protocol/readable non-regression 的证明目标。
- [ ] 2.6 审计 `docs/schemas/` 和 `docs/examples/` 中仍引用旧 config command、裸 `options.<key>` 或旧 config source shape 的材料；按 adapter-id path 迁移同步更新并验证。
- [ ] 2.7 Owner 主规范退出门禁：确认 CLI、navigation-input-resolution、architecture、adapter-contract、adapter docs、schema/example 和 testing owner 文档之间没有同一目标内部冲突。

## 3. 实现前证明目标与测试骨架

- [ ] 3.1 为旧 `config get|set|unset|list` 子命令被拒绝、`config inspect` 只读 source status、当前可解析参数事实展示和不修改 config file 建立最小 CLI 测试或测试计划。
- [ ] 3.2 为参数汇总 projection 建立 tests，证明 CLI/input 与 config-source projection 复用同一 owner-provided facts，且不会在 core config command 中重新定义字段语义。
- [ ] 3.3 为 adapter-id namespace 建立 tests，证明同名 option key 在不同 adapter id namespace 下保持 deterministic，裸 `options.<key>` 按普通 unknown/invalid config path 处理。
- [ ] 3.4 为 selected adapter/operation resolution 建立 tests，证明 navigation 只消费 selected adapter namespace，其它已知 adapter namespace 不 forward 给 selected adapter handler。
- [ ] 3.5 为 `outline.mode_rules[]`、`outline.auto_full_read.thresholds[]` 建立 parity tests，先证明现有 owner-specific validation 能否覆盖 source path、unknown item key、required member、typed value 和 navigation resolution diagnostics。
- [ ] 3.6 只有 3.5 证明现有路径不足时，才为 typed-fields 最小 compound helper 建立单元测试，覆盖 processing-path lookup、array item/object member metadata、nested failure path 和 duplicate path。

## 4. Slice A: Core Config Inspect Surface

- [ ] 4.1 删除 `docnav config get|set|unset|list` parser/model/command surface 和相关 help/docs tests，新增单一 `docnav config inspect` 只读 command。
- [ ] 4.2 实现 config inspect source status 输出，覆盖 selected project/user config path、origin、exists/missing、load state、JSON/config validation issue 和可解析 source summary。
- [ ] 4.3 让 config inspect 列出当前输入可解析出的参数事实，但不声明 adapter dispatch、不构造 operation request、不修改 config file。
- [ ] 4.4 更新 config inspect help/docs/tests，证明 accepted inspect surface 与 source inspection 输出一致。
- [ ] 4.5 运行 Slice A 的局部 CLI/Rust 测试和 OpenSpec validation。

## 5. Slice B: 参数汇总与 Adapter-id Config Path

- [ ] 5.1 在参数汇总边界中建立或暴露 config-source projection，复用 existing `ParameterResolutionPipeline`、navigation common fields、outline mode config fields、core-owned runtime fields 和 `AdapterOptionSpec` declarations。
- [ ] 5.2 实现 `options.<adapter-id>.<option-key>` path 解析、adapter registry lookup 和 adapter-local declaration conflict；裸 `options.<key>` 只按普通 unknown/invalid config path 处理。
- [ ] 5.3 更新 hard-coded typed-field/config paths、config key registry、config store read validation 和 smoke fixtures，使 persistent config-source path 使用 adapter-id namespace。
- [ ] 5.4 让 config inspect、config source validation 和 navigation input resolution 消费同一参数汇总 projection，不在 core config command 中复制 adapter-owned value kind、range、default 或 operation applicability。
- [ ] 5.5 让 navigation 在 selected adapter/operation resolution 阶段消费对应 adapter namespace 的 config values；其它已知 adapter namespace 不 forward 给 selected adapter handler。
- [ ] 5.6 运行 Slice B 的 adapter-id namespace、parameter projection、schema/example 和 navigation resolution parity tests。

## 6. Slice C: 数组配置校验审计与按需 Typed-fields 扩展

- [ ] 6.1 审计现有 `outline` owner validation、config key registry 和 config store shape validation，确认 `outline.mode_rules[]`、`outline.auto_full_read.thresholds[]` 的 source path、unknown item key、required member、regex/enum/positive integer diagnostics 是否已满足 inspect/read/resolution parity。
- [ ] 6.2 若 6.1 已满足 parity，保留 owner-specific validation，并用 tests 记录该 owner boundary；不得为了抽象一致性强行迁移到 typed-fields。
- [ ] 6.3 若 6.1 不满足 parity，在 `docnav-typed-fields` 中增加或暴露当前 config-source subset 所需的最小 compound metadata helper，覆盖 object members、array item schema、nested processing path 和 source path。
- [ ] 6.4 若引入 typed-fields helper，只迁移能被 helper 清楚表达的当前数组配置；保留的 owner-specific policy 必须有测试、注释、范围和移除条件。
- [ ] 6.5 运行 Slice C 的 typed-fields 或 owner-specific validation tests、config file direct edit/read tests 和 nested shape diagnostics tests。

## 7. 集成验证与交付审计

- [ ] 7.1 增加或补齐 core config inspect tests，覆盖 source status、missing/unreadable/invalid/non-object config source、invalid positive integer、invalid output enum、invalid native option type/range、unknown adapter id、裸 options path unknown、旧 `get|set|unset|list` 子命令被拒绝。
- [ ] 7.2 增加或补齐 config file direct edit/read tests，覆盖 unknown field、invalid typed value、unsupported selected adapter option、adapter-id native option path 和非结构化全文读取数组配置。
- [ ] 7.3 增加或补齐 protocol/readable non-regression tests，证明本 change 不改变 document operation `protocol-json`、`readable-json`、readable-view stdout 或 linked adapter handler payload。
- [ ] 7.4 运行受影响 Rust tests、OpenSpec validation 和局部 docs/schema/example checks；若同步修改 docs、schema、examples 或跨 crate 行为，运行 `bun run verify:docnav-workspace`。
- [ ] 7.5 用局部 diff 审计实现是否只触及 config inspect、parameter aggregation backed config validation、adapter-id config path 相关 docs、OpenSpec、tests 和代码，且没有修改无关 protocol/output/adapter handler behavior。
- [ ] 7.6 抽查 `options.<adapter-id>.<option-key>` 配置读取，确认同名 option 可在不同 adapter namespace 下共存、裸 options path 按普通 unknown/invalid 处理、selected adapter validation 和 diagnostic details 与 specs 一致。
- [ ] 7.7 记录最终验证命令、结果和任何未覆盖风险，再进入归档或后续实现审计。
