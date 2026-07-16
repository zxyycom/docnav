本 task list 的目标是实现 `ProtocolJson` 与 `Rendered(RenderStrategy)` 两路径 output contract；当前文档只在 `openspec/changes/unify-output-with-injected-rendering/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## 1. 阻塞级实现前审计

- [ ] 1.1 审计 proposal、design、五个 capability delta specs 和 tasks 是否围绕“两条 output 路径、code-only renderer injection、默认 `readable-view`、单一 protocol machine contract”这一主承诺。
- [ ] 1.2 审计 owner 边界：output composition 注入 renderer，adapter/navigation 只产出 outcome，core/CLI 只选择 mode 和映射 process behavior。
- [ ] 1.3 审计 `RenderInput` 未形成 serialized shadow contract，`ProtocolJson` 完全绕过 renderer，renderer failure 无 partial stdout 或隐式 renderer 切换。
- [ ] 1.4 确认当前 change 只包含未审核临时 artifacts，旧 `add-adapter-readable-view-rendering` 已删除，主 specs、docs、schema、examples 和实现尚未在提案阶段修改。
- [ ] 1.5 与 `derive-document-cli-options-from-fields` 保持 owner 边界：本 change 修改 canonical output field facts，对方只投影这些 facts；两者不建立实施顺序门禁，后合并的一方处理普通代码冲突。重基或暂停 `interactive-outline-selection`、`implement-docnav-mcp-bridge`、`add-outline-preview-skim-pack`、`add-obvious-result-auto-read` 和 `explore-operation-composition` 中的旧三模式假设。
- [ ] 1.6 确认 capability mapping 完整、`## Open Questions` 无待确认项；1.1-1.5 完成前不得执行后续实现任务。

## 2. Owner 文档与验证材料

- [ ] 2.1 按 `docs/navigation.md` owner 路径更新 `docs/output.md` 与 `docs/cli.md`，定义两路径模型、renderer dependency、channel ownership 和 failure behavior。
- [ ] 2.2 按 `docs/testing.md` 与 `docs/testing/case-maintenance.md` 记录 protocol、renderer、CLI、logging 和 release 的证明目标。
- [ ] 2.3 更新 schema、examples、fixtures、goldens、conformance materials 和索引，删除不再拥有 public contract 的 readable JSON materials。
- [ ] 2.4 更新 MCP、interactive 和 composition 相关计划，使结构化消费使用 `protocol-json`，presentation 使用 linked renderer contract。

## 3. Output 与 core 实现

- [ ] 3.1 将 shared document output model 收敛为 `ProtocolJson` 与 `Rendered(RenderStrategy)`，并将 CLI mode declaration 收敛为 `readable-view` 与 `protocol-json`。
- [ ] 3.2 定义 code-only `RenderStrategy`、`RenderInput`、immutable `RenderContext` 和 `RenderFailure`，覆盖 success outcome 与 primary diagnostic。
- [ ] 3.3 将现有 readable rendering 重构为内置 `readable-view` renderer，保持其 owner-declared framing 与 unstructured outline behavior。
- [ ] 3.4 更新 output orchestration：protocol path 直接序列化 envelope；rendered path 完成渲染后一次性写入完整 UTF-8 text。
- [ ] 3.5 实现 renderer failure boundary，覆盖 panic/error、空 stdout、稳定 stderr/exit mapping 和 no renderer switching。
- [ ] 3.6 在 core composition root 注入默认 renderer，并允许其它 linked code caller 直接构造带自定义函数的 `Rendered` plan。
- [ ] 3.7 删除 readable JSON mode、serializer、public DTO/schema bindings 和专属 validation helpers；output field 只保留当前两个合法值。

## 4. 自动化证明

- [ ] 4.1 添加 renderer/output tests，覆盖默认与自定义 renderer、success/failure input、exact UTF-8 text、无额外 framing、无 partial stdout 和 renderer failure。
- [ ] 4.2 添加 protocol tests，证明 success/failure envelope、result/error、ref、pagination 和 stdout 单一 JSON value 不受 renderer 影响。
- [ ] 4.3 添加 core CLI smoke，覆盖 omitted output、explicit `readable-view`、`protocol-json`、early PlainText 和普通 invalid output validation。
- [ ] 4.4 更新 diagnostics 与 invocation logging tests，证明 primary diagnostic identity、renderer input/output、protocol stdout 和独立 log sink 保持隔离。
- [ ] 4.5 更新 release package 与 schema/example validation，证明 package 默认 renderer 和剩余 public JSON contracts 可验证。

## 5. 验证与交付

- [ ] 5.1 运行范围匹配的 Rust format、static checks、unit、integration 和 CLI smoke。
- [ ] 5.2 运行 `openspec validate unify-output-with-injected-rendering --type change --json --strict --no-interactive` 与 repository OpenSpec validation。
- [ ] 5.3 运行 schema/example validation 和 `bun run verify:docnav-workspace`，覆盖跨 Rust、docs、OpenSpec、schema、logging 与 release 边界。
- [ ] 5.4 用局部 diff 与 filtered `rg` 确认只改目标 owner，document output 不再发布 readable JSON contract，protocol 与 PlainText owner 未扩大。
