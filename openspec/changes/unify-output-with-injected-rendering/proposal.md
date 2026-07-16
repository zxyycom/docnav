## Why

Docnav 当前为同一 document outcome 维护 `protocol-json`、`readable-json` 和 `readable-view` 三种 public output shape。`readable-json` 与 protocol envelope 同时承担 machine-readable contract，扩大了 schema、projection、consumer mapping 和验证维护面；`readable-view` 又被绑定到这套 serialized readable DTO，renderer 无法作为独立 presentation policy 复用。

本 change 将稳定机器接口收敛为 `protocol-json`，并让 presentation 通过 linked code 提供的 renderer 完成。Core CLI 继续以 `readable-view` 作为默认阅读输出，但不再发布第二套 readable JSON contract。

## Scope

- Internal document output paths：`ProtocolJson` 与 `Rendered(RenderStrategy)`。
- Public CLI/config values：`readable-view` 与 `protocol-json`；省略 output 或选择 `readable-view` 时，core CLI 始终注入内置 renderer。
- Linked code API：直接调用 shared output API 的代码可以构造 `Rendered` 并注入自定义 renderer；renderer identity 不进入 CLI、config、environment、manifest、plugin 或 subprocess contract。
- Stable owners：output orchestration 拥有 path selection、stdout/stderr、render failure 和 process mapping；protocol、diagnostic、adapter result、ref 与 pagination 继续由现有 owner 定义。
- Breaking boundary：删除 `readable-json` mode、serialized readable DTO 及其 schema、examples、fixtures、goldens 和 validation branches。

## What Changes

1. 将 shared document output model 收敛为 `ProtocolJson` 与 `Rendered(RenderStrategy)`，并将 public document output values 收敛为 `readable-view` 与 `protocol-json`。
2. 定义 code-only renderer contract：消费完成的 typed success outcome 或 primary `DiagnosticRecord` 与 immutable context，返回完整 UTF-8 text 或 `RenderFailure`。
3. 让 rendered path 在内存渲染成功后一次性提交文本；可恢复渲染失败映射为 output-owned `output_render_failed`，保持 stdout 为空且不切换 renderer。
4. 让 `protocol-json` 完全绕过 renderer，并保持现有 envelope、result/error、ref 和 pagination contract。
5. 删除 `readable-json` public contract，并迁移依赖它的 active plans、docs、schemas、examples、fixtures 和 tests。

## Done When

1. Document CLI/config 只接受 `readable-view` 与 `protocol-json`；core CLI 的 `readable-view` 始终使用内置 renderer。
2. Linked code 可以直接注入自定义 renderer，且 public input、serialized metadata 与 adapter definition 中不存在 renderer implementation identity。
3. Protocol success/failure envelope 不受 renderer availability 或 behavior 影响；rendered success 精确提交一个完整 text value。
4. 同一个 primary diagnostic 在 protocol serializer 与 renderer input 中保持 canonical identity；`RenderFailure` 统一产生 `output_render_failed`、empty stdout 和既定 internal failure exit mapping。
5. Runtime、docs 和 validation materials 不再发布或消费 `readable-json` contract，依赖旧 contract 的 active changes 在恢复实现前完成重基。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `output-contract`：定义 protocol/rendered 两路径、renderer dependency、text commit 和 failure boundary。
- `core-cli`：将 document output values 收敛为 `readable-view` 与 `protocol-json`，并为 CLI rendered path 注入内置 renderer。
- `diagnostics-contract`：保持 primary diagnostic identity，并定义通用 output render failure identity。
- `invocation-logging`：保持 protocol stdout、renderer input/output 和独立 log sink 隔离。
- `release-artifacts`：从 package 验证 protocol output 与内置 rendered output。

## Impact

- 影响 `docnav-output`、`docnav-readable`、core CLI output composition、diagnostic boundary code 和相关 tests。
- 影响 `docs/architecture.md`、`docs/output.md`、`docs/cli.md`、testing owner materials、schema/example/fixture/golden 索引、invocation logging 和 release verification。
- `derive-document-cli-options-from-fields` 只投影 canonical output field facts，与本 change 无实施顺序依赖。
- 依赖 `readable-json` 或 typed readable DTO 的 active changes 保持暂停，并在恢复实施前改用 `protocol-json` 或 linked renderer API。
- `protocol-contract`、adapter operation contract、ref、pagination、routing 和 parsing ownership 保持不变。
