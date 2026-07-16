## 1. 已完成的实现前审计

- [x] 1.1 确认 proposal、design、五个 capability delta specs 和 tasks 共同兑现“两条 output paths、code-only renderer injection、core 默认 `readable-view`、单一 protocol machine contract”，并移除无关的 config surface delta。
- [x] 1.2 确认 owner 边界：adapter/navigation 只产出 outcome，diagnostics 拥有 identity，protocol 拥有 machine envelope，linked composition 提供 renderer，output orchestration 拥有 channels/failure，core CLI 只映射 mode 与 exit behavior。
- [x] 1.3 确认 `RenderInput` 仅为 private typed code contract，`ProtocolJson` 不构造或调用 renderer，rendered text 在成功后一次性提交，returned `RenderFailure` 不产生 partial stdout 或 renderer fallback。
- [x] 1.4 通过 Git history 与路径检查确认旧 `add-adapter-readable-view-rendering` 已删除，提案阶段只修改 change artifacts，尚未修改主 specs、docs、schema、examples 或 runtime code。
- [x] 1.5 确认 `derive-document-cli-options-from-fields` 只投影 canonical output field facts；依赖 `readable-json` 的 `interactive-outline-selection`、`implement-docnav-mcp-bridge`、`add-outline-preview-skim-pack`、`add-obvious-result-auto-read` 和 `explore-operation-composition` 均未开始实现，并保持暂停直到重基到两路径 contract。
- [x] 1.6 确认 capability mapping 覆盖 output、core CLI、diagnostics、invocation logging 与 release artifacts；`## Open Questions` 无待确认项，可以进入 owner docs 与实现阶段。

## 2. Owner 文档与验证材料

- [ ] 2.1 更新 `docs/architecture.md`、`docs/output.md`、`docs/cli.md` 与 `docs/navigation.md`，定义两路径模型、CLI-to-renderer mapping、linked dependency、channel ownership 和 recoverable failure behavior。
- [ ] 2.2 按 `docs/testing.md` 与 `docs/testing/case-maintenance.md` 更新 protocol、built-in/custom renderer、CLI、diagnostics、logging 和 release 的证明目标与 case mapping。
- [ ] 2.3 更新 usage docs、schema、examples、fixtures、goldens、conformance materials 和索引，删除不再拥有 public contract 的 readable JSON materials。
- [ ] 2.4 重基 `interactive-outline-selection`、`implement-docnav-mcp-bridge`、`add-outline-preview-skim-pack`、`add-obvious-result-auto-read` 和 `explore-operation-composition`：结构化消费使用 `protocol-json`，presentation 使用 built-in 或 linked renderer contract。

## 3. Output 与 core 实现

- [ ] 3.1 将 shared document output model 收敛为 `ProtocolJson` 与 `Rendered(RenderStrategy)`，并将 canonical CLI output field 收敛为 `readable-view` 与 `protocol-json`。
- [ ] 3.2 定义 code-only `RenderStrategy`、`RenderInput`、immutable `RenderContext` 和 `RenderFailure`，覆盖 typed success outcome 与 primary diagnostic，不暴露 serialized helper view。
- [ ] 3.3 将现有 readable rendering 重构为内置 `readable-view` renderer，保持 owner-declared framing、unstructured outline 和 primary diagnostic presentation。
- [ ] 3.4 更新 output orchestration：protocol path 直接序列化 envelope；rendered path 完成内存渲染后原样写入一个完整 UTF-8 text value。
- [ ] 3.5 将 returned `RenderFailure` 映射为 output-owned `output_render_failed`，保持 empty stdout、bounded stderr diagnostic、internal failure exit mapping 和 no renderer switching。
- [ ] 3.6 在 core composition root 为省略 output/`readable-view` 注入内置 renderer，并允许其它 linked code caller 直接构造带自定义 renderer 的 `Rendered` plan。
- [ ] 3.7 删除 readable JSON mode、serializer、public DTO/schema bindings 和专属 validation helpers；runtime 不保留 alias、fallback 或 parallel branch。

## 4. 自动化证明

- [ ] 4.1 添加 renderer/output tests，覆盖内置与自定义 renderer、success/diagnostic input、exact UTF-8 text、private helper boundary、returned failure、无额外 framing、无 partial stdout 和无 fallback。
- [ ] 4.2 添加 protocol tests，证明 success/failure envelope、result/error、ref、pagination 和 stdout 单一 JSON value 不受 renderer availability 或 behavior 影响。
- [ ] 4.3 添加 core CLI smoke，覆盖 omitted output、explicit `readable-view`、`protocol-json`、early PlainText、普通 invalid output 和 CLI 无 renderer identity。
- [ ] 4.4 更新 diagnostics 与 invocation logging tests，证明 primary diagnostic identity、`output_render_failed`、renderer input/output、protocol stdout 和独立 log sink 保持正确归属。
- [ ] 4.5 更新 release package 与 schema/example validation，证明 package 内置 renderer 和剩余 public JSON contracts 可验证。

## 5. 验证与交付

- [ ] 5.1 运行范围匹配的 Rust format、static checks、unit、integration 和 CLI smoke。
- [ ] 5.2 运行 `openspec validate unify-output-with-injected-rendering --type change --json --strict --no-interactive` 与 repository OpenSpec validation。
- [ ] 5.3 运行 schema/example validation 和 `bun run verify:docnav-workspace`，覆盖 Rust、docs、OpenSpec、schema、logging 与 release 边界。
- [ ] 5.4 用局部 diff 与 filtered `rg` 确认只改目标 owner，runtime 和 validation materials 不再发布 readable JSON contract，protocol、PlainText 与 adapter owner 未扩大。
