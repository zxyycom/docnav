本 tasks 只在 `openspec/changes/replace-config-commands-with-inspect/` 下形成 change-stage checklist；实现任务执行前必须先完成 config inspect surface 与 metadata-backed validation 方案审计门禁。

## 1. 阻塞级审计门禁

- [ ] 1.1 审计 proposal、design、specs 和 tasks 是否都围绕“用单一只读 `docnav config inspect` 替换旧 config 子命令，并让配置读取校验与配置检查复用 owner-provided config metadata”这一核心目标；审计未完成前不得执行任何实现任务。
- [ ] 1.2 审计 capability ID 是否只复用 `core-cli`、`navigation-input-resolution`、`typed-fields` 和 `adapter-contract`，且没有把 change name 或宽泛 config umbrella 当作新 capability。
- [ ] 1.3 审计提案阶段改动是否只修改 `openspec/changes/replace-config-commands-with-inspect/` 下的未审核临时 artifacts，且没有修改现有 specs、docs、schemas、examples、测试或实现代码。
- [ ] 1.4 审计 `design.md` 是否没有未回答开放问题，并确认单一 source-scoped `config inspect` surface、`options.<adapter-id>.<option-key>` canonical path、裸 `options.<key>` 不兼容策略、diagnostic mapping 和 compound config metadata 迁移边界都已进入 specs 或后续任务。
- [ ] 1.5 审计本 change 是否不改变 raw protocol envelope、readable output wrapper、linked adapter handler payload 或 adapter-owned native option namespace。

## 2. 规范同步

- [ ] 2.1 更新 `docs/cli.md`，删除 `docnav config get|set|unset|list` 长期 surface，固定单一 source-scoped `docnav config inspect` 输出、`options.<adapter-id>.<option-key>` 路径和破坏式迁移边界。
- [ ] 2.2 更新 `docs/navigation-input-resolution.md`，声明 config metadata projection 如何聚合 common fields、outline fields 和 adapter-id namespaced declarations，并如何驱动 config source validation。
- [ ] 2.3 更新 `docs/architecture.md` 和 `docs/adapter-contract.md`，明确 typed-fields 仍只拥有字段事实与 projection，adapter option semantics 仍由 adapter declaration 拥有。
- [ ] 2.4 更新 `docs/testing.md`、`docs/testing/case-maintenance.md` 或 case ledger 中对应条目，写明 config inspect/read/resolution parity、adapter-id option namespace、nested shape validation、旧 config 子命令移除和 protocol/readable non-regression 的证明目标。

## 3. Typed Metadata Projection

- [ ] 3.1 在 `docnav-typed-fields` 中增加或暴露按 processing id + structured path 查询 field metadata 的 projection helper，覆盖 field identity、value kind、constraints、nullability/default、compound node kind、object members、array item schema 和 source path。
- [ ] 3.2 增加 typed-field candidate JSON validation helper，返回 canonical typed value 或带 field identity、processing path、nested received path、received kind 和 constraint/shape reason 的 validation failure；确认 CLI lexical shorthand 不进入 typed-fields。
- [ ] 3.3 扩展 typed-fields 的 config-source subset，支持 array、object 和 nested structure validation，覆盖 `outline.mode_rules[]`、`outline.auto_full_read.thresholds[]` 等现有 nested config；若短期保留 owner-specific policy，必须有测试、注释、范围和移除条件。
- [ ] 3.4 为 processing-path lookup、missing path、duplicate path、candidate scalar failure、array item failure、object member failure 和 nested shape failure 增加 typed-fields 单元测试。

## 4. Navigation 和 Adapter Metadata 集成

- [ ] 4.1 在 `docnav-navigation` 中建立 config metadata aggregation，复用 navigation common fields、outline mode config fields 和 adapter-id namespaced `AdapterOptionSpec` field declarations。
- [ ] 4.2 让 navigation config source validation 使用 config metadata projection 判断 unknown field、expected object/array shape、unknown adapter id、undeclared `options.<adapter-id>.*` 和 typed value failures。
- [ ] 4.3 让 adapter-id native option metadata 支持 config source validation 与 navigation input resolution，并保持 `config inspect` 输出聚焦配置来源事实。
- [ ] 4.4 增加 tests，证明 selected adapter option validation、unsupported selected option、same adapter-id path parity、unknown adapter id 和 source-attributed diagnostics。

## 5. Core Config Inspect 和 Store 迁移

- [ ] 5.1 删除 `docnav config get|set|unset|list` parser/model/command surface 和相关 help/docs tests，新增单一 `docnav config inspect` 只读 command。
- [ ] 5.2 实现 config inspect source status 输出，覆盖 selected project/user config path、origin、exists/missing、load state、JSON/config validation issue 和可解析 source summary。
- [ ] 5.3 实现 `options.<adapter-id>.<option-key>` path 解析、adapter registry lookup 和 adapter-local declaration conflict；裸 `options.<key>` 只按普通 unknown/invalid config path 处理。
- [ ] 5.4 迁移 config store read validation，尽量用 config metadata 取代手写 shape/value registry；保留的 owner-specific 校验必须有注释说明原因、范围和移除条件。
- [ ] 5.5 更新 config inspect help/docs/tests，证明 accepted inspect surface 与 source inspection 输出一致。

## 6. 测试与验证

- [ ] 6.1 增加 core config inspect tests，覆盖 source status、missing/unreadable/invalid/non-object config source、invalid positive integer、invalid output enum、invalid native option type/range、unknown adapter id、裸 options path unknown、旧 `get|set|unset|list` 子命令被拒绝。
- [ ] 6.2 增加 config file direct edit/read tests，覆盖 unknown field、invalid nested object、invalid typed value、unsupported selected adapter option 和 outline config array fields。
- [ ] 6.3 增加 parity tests，证明同一 config value 在 `docnav config inspect` / config source validation 与 navigation input resolution 中使用同一 metadata 得到一致结果。
- [ ] 6.4 增加 protocol/readable non-regression tests，证明本 change 不改变 document operation `protocol-json`、`readable-json`、readable-view stdout 或 linked adapter handler payload。
- [ ] 6.5 运行受影响 Rust tests、OpenSpec validation 和局部 docs/schema/example checks；若同步修改 docs、schema、examples 或跨 crate 行为，运行 `bun run verify:docnav-workspace`。

## 7. 交付审计

- [ ] 7.1 用局部 diff 审计实现是否只触及 config inspect、metadata-backed config validation 相关 docs、OpenSpec、tests 和代码，且没有修改无关 protocol/output/adapter handler behavior。
- [ ] 7.2 抽查 `options.<adapter-id>.<option-key>` 配置读取，确认同名 option 可在不同 adapter namespace 下共存、裸 options path 按普通 unknown/invalid 处理、selected adapter validation 和 diagnostic details 与 specs 一致。
- [ ] 7.3 记录最终验证命令、结果和任何未覆盖风险，再进入归档或后续实现审计。
