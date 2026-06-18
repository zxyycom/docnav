本 tasks 目标是列出共享 crate 所有权重构的文档先行和实现步骤，后续实现从主规范和验证材料同步开始。

## 1. 主规范和验证材料先行

- [ ] 1.1 同步 `docs/architecture.md` 中共享 crate 职责、依赖方向和 owner 表述，明确本 change 主目标是共享 crate ownership 去重，并记录 `docnav-json-io` 位于 document output 编排下层。
- [ ] 1.2 同步 `docs/cli.md` 和 `docs/output.md` 中 direct CLI warning、document output mode、protocol-json stdout purity、readable warning placement、非文档输出边界和低层 JSON writer 复用边界。
- [ ] 1.3 同步 `docs/protocol.md` 和 `docs/adapter-contract.md` 中 protocol decode pipeline、adapter `invoke` 严格校验、adapter direct CLI 宽松 argv 和 manifest/probe 边界。
- [ ] 1.4 同步 `docs/testing.md`、schema/example/fixture 说明和 smoke case 清单，明确需要验证的 public surface 和 helper 迁移边界。
- [ ] 1.5 主规范和验证材料同步完成前不得开始 crate 新增或代码迁移；发现规范与 OpenSpec delta 不一致时，先修正文档再实现。

## 2. 行为刻画测试

- [ ] 2.1 为 core CLI ignored argv、adapter candidate warning、readable-json warning、readable-view warning 和 protocol-json stderr warning 增加或确认现有 characterization coverage。
- [ ] 2.2 为 adapter direct CLI unknown flag、extra positional、unused known/native flag、protocol-json stderr warning、manifest/probe warning stderr 边界增加或确认现有 characterization coverage。
- [ ] 2.3 为 Markdown adapter text/entry pagination、Unicode 字符预算、oversized entry consumption、next page 和 truncation behavior 增加或确认现有 characterization coverage。
- [ ] 2.4 为 protocol request/response、manifest、probe 的 schema invalid、deserialize invalid 和 semantic invalid 路径增加或确认现有 characterization coverage。

## 3. Diagnostics 和 CLI argv helper

- [ ] 3.1 新增 `docnav-diagnostics` crate，定义稳定 warning envelope、`WarningId` opaque newtype、共享 warning id 常量、warning id/effect/details、argv warning constructors 和 stderr warning text formatter。
- [ ] 3.2 迁移 core CLI warning construction 和 protocol-json stderr warning text 到 `docnav-diagnostics`，保持 warning id、effect、details 和 stdout/stderr 边界不变。
- [ ] 3.3 迁移 adapter SDK direct CLI warning construction 和 stderr warning text 到 `docnav-diagnostics`，保持 adapter direct machine command stdout schema 不变。
- [ ] 3.4 新增 `docnav-cli-args` crate，抽取 direct CLI loose argv token scanner，输入为 caller-provided command context 和 known value flag metadata，输出为 ignored token diagnostics facts。
- [ ] 3.5 迁移 core CLI 和 adapter SDK direct CLI 的 loose argv 扫描到 `docnav-cli-args`，保留各自 typed argument validation、defaults 和 business request construction。

## 4. JSON IO 和 document output 编排

- [ ] 4.1 新增 `docnav-json-io` crate，定义 JSON value serialization、newline writing 和 serialization/write failure plumbing，且不拥有 schema、protocol/readable wrapper、warning、output mode 或 exit code policy。
- [ ] 4.2 新增 `docnav-output` crate，使其依赖 `docnav-protocol`、`docnav-readable`、`docnav-diagnostics` 和 `docnav-json-io`，且不依赖 `docnav` core 或 `docnav-adapter-sdk`。
- [ ] 4.3 在 `docnav-output` 中定义 document-only output facade、document output mode dispatch、readable/protocol success rendering、stable error rendering、warning injection 和 stdout/stderr channel decisions。
- [ ] 4.4 将 core CLI document operation output dispatch 迁移到 `docnav-output`，保持 `readable-view`、`readable-json` 和 `protocol-json` 的 documented shape 与通道不变。
- [ ] 4.5 将 adapter SDK direct document operation output dispatch 迁移到 `docnav-output`，保留 manifest、probe、invoke 和 help 的既有 adapter contract 边界；非文档 machine output 可复用 `docnav-json-io` 或 diagnostics helper，但不通过 `docnav-output` 编排。
- [ ] 4.6 将 `docnav-readable` 收敛为 readable payload/value helper、`ReadableViewKind`、renderer config、readable-view block renderer 和 conformance vectors，不让其拥有 output mode dispatch。

## 5. Adapter SDK paging 和 protocol helper

- [ ] 5.1 在 `docnav-adapter-sdk` 中新增 format-neutral paging helper，覆盖 text/entry pagination、character budget、next page 和 truncation mechanics。
- [ ] 5.2 将 Markdown adapter 现有分页 mechanics 迁移到 SDK paging helper，保留 Markdown parser、heading、ref generation/parsing 和 display semantics 在 `docnav-markdown` 内。
- [ ] 5.3 在 `docnav-protocol` 中新增 request id helper，并迁移 core invoke 和 output error 路径中格式相近的 request id generation。
- [ ] 5.4 在 `docnav-protocol` 中新增 `Value -> schema validate -> deserialize -> semantic validate` decode pipeline helper，保持 caller-owned error attribution、field path、diagnostic text 和 exit behavior。
- [ ] 5.5 增加 `StableErrorCode` 分类 helper，供 core 和 adapter SDK 映射到各自 exit code enum；不得合并 core 和 adapter 的 concrete exit code enum。

## 6. 最终验证

- [ ] 6.1 运行相关 Rust unit tests、adapter/core smoke tests 和 schema/example validation，确认 helper 迁移未改变 public surface。
- [ ] 6.2 对跨 crate 交付运行 `pnpm run verify:docnav-workspace` 或当时主规范指定的等价 workspace verifier。
- [ ] 6.3 审查最终 diff，确认只包含本 change 范围内的 crate、docs、tests 和 validation material 修改，且没有引入 path utility、process runner 或 adapter boundary crate。
