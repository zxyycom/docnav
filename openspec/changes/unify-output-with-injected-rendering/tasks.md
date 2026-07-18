## 1. 已完成的设计收敛

- [x] 1.1 确认 `ProtocolJson` 与 `Rendered(RenderStrategy)` 共同消费既有 `ProtocolResponse`，不新增 outcome/context model。
- [x] 1.2 确认 public document output values 为 `readable-view` 与 `protocol-json`，renderer implementation 只由 linked code 提供。
- [x] 1.3 确认 capability scope 只包含 `output-contract` 与 `core-cli`，并以 hard cutover 删除 `readable-json`。

## 2. Owner 文档与验证材料

- [x] 2.1 更新 `docs/architecture.md`、`docs/output.md`、`docs/cli.md` 与 `docs/navigation.md`，定义统一 `ProtocolResponse` 输入、两条 output paths 和 CLI mapping。
- [x] 2.2 删除 public `readable-json` schema/examples/fixtures/validators，并让 built-in renderer conformance 从 `ProtocolResponse` 覆盖最终 `readable-view` text。
- [x] 2.3 按 testing owner 文档更新 output、CLI、protocol isolation 和 migration case mapping。

## 3. Output 与 core 实现

- [x] 3.1 在 shared output 中定义 `ProtocolJson`、`Rendered(RenderStrategy)`、`RenderStrategy(&ProtocolResponse)` 和 `RenderFailure`。
- [x] 3.2 将现有 readable rendering 包装为接收 `ProtocolResponse` 的内置 renderer，保留最终 `readable-view` framing 和 presentation。
- [x] 3.3 让 document success 和 failure 在进入 output plan 前都形成 `ProtocolResponse`；protocol path 直接序列化，rendered path 调用选定 renderer 并原样写文本。
- [x] 3.4 从 CLI/config accepted values、runtime branches 和 public validation bindings 删除 `readable-json`，仅保留内置 renderer 需要的 private helpers。

## 4. 自动化证明

- [x] 4.1 添加 shared output tests，覆盖 success/failure `ProtocolResponse`、built-in/custom renderer、exact text、`RenderFailure` before stdout、no fallback 和独立 writer failure。
- [x] 4.2 添加 core CLI tests，覆盖 omitted output、explicit `readable-view`、CLI/config-selected `protocol-json`、early document failure 和 removed `readable-json` rejection。
- [x] 4.3 更新 readable conformance 与 protocol integration tests，证明 built-in text contract 保持且 protocol envelope/schema 不变。

## 5. 验证与交付

- [x] 5.1 运行范围匹配的 Rust format、static checks、unit、integration 和 CLI smoke。
- [x] 5.2 运行 OpenSpec、docs、schema/example validation 和 `bun run verify:docnav-workspace`。
- [x] 5.3 用局部 diff 和 filtered search 确认 Current runtime/docs/validation 不再发布 `readable-json`，且修改范围未扩到 protocol、diagnostics、logging 或 release owner。
