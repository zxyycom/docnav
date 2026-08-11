# Proposal

本计划在核心契约稳定且产品方向恢复该工作后，为 TTY 用户交付 `docnav outline <path> --interactive` 的多选读取流程；当前按[核心契约稳定后再扩张接入与交互面](../../docs/decisions/product-direction/stabilize-core-before-entrypoint-expansion.md)暂停，而不是退回只有方向的 Draft。

## Why

人类用户现在需要从 outline 手动复制 ref 再调用 read，连续比较多个章节时操作成本较高。该问题适合由 core CLI 编排解决，但 terminal UI、取消、非 TTY 和多结果呈现不应反向改变 adapter、ref 或 machine protocol。

## Outcome

产品恢复后，TTY 用户可从既有 outline entries 中选择一个或多个 ref，并按选择顺序复用现有 read 语义查看内容；非 TTY、machine output、取消、空选择和空 outline 都有明确且可测试的行为。

## Scope

- 纳入：core CLI `outline --interactive`、TTY 检查、prompt-style 多选、entry-to-ref mapping、顺序 read、human output、取消与空状态、跨平台终端验证。
- 不纳入：adapter direct CLI、adapter/protocol/ref shape 变更、machine-readable batch response、树形折叠、preview pane、鼠标、复杂快捷键或 full-screen TUI。
- 当前暂停只阻止实施；恢复时先重审 Current CLI/output/ref 基线并关闭依赖与最终 UX 门禁。

## Success Criteria

- 普通 `outline` 行为不变；只有显式 `--interactive` 且 stdin/stdout 满足终端条件时进入 UI。
- 选项 display 只用于展示，opaque ref 是后续 read 的唯一业务输入；多个选择按确认顺序读取。
- `protocol-json` 与 interactive 互斥，非 TTY 返回稳定 invalid request；取消、空选择和空 outline 不误触发 read。
- Rust tests、可测试交互 seam、Windows terminal smoke、owner 文档和 workspace 验证通过。

## Affected Owners

- [CLI](../../docs/cli.md)和 [Navigation Input Resolution](../../docs/navigation-input-resolution.md)：实施期间作为参数、TTY 门禁和 outline-to-read 编排的 Current 基线；行为证据成立后再同步实际新增的 Current surface。
- [输出模式](../../docs/output.md)：实施期间作为 human-only 展示和 protocol-json 边界的 Current 基线；[Ref](../../docs/ref-contract.md)始终只是不变的交接约束。
- 本 design 登记 [CLI](../../docs/cli.md)、[Navigation Input Resolution](../../docs/navigation-input-resolution.md)、[输出模式](../../docs/output.md)、[测试策略](../../docs/testing.md)和[发布包验证](../../docs/testing/release.md)的预期 delta；只有实现与行为证据通过后，才把相应 delta 写成 Current。
