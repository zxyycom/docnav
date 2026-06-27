本 proposal 起草 `interactive-outline-selection` 的目标：为 `docnav outline <path>` 增加面向人类的交互式选择流程，使用户无需手动复制 ref 即可选择 outline 条目并读取内容；当前 change 只在 `openspec/changes/interactive-outline-selection/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## Why

当前 `outline -> ref -> read` 流程对 AI 和脚本友好，但人类用户在终端中需要手动复制 outline entry 的 ref，再调用 `read`。当用户需要比较多个章节、连续读取多个条目或从大型文档中筛选内容时，这个复制粘贴步骤会显著降低 CLI 的可用性。

Docnav 已经拥有稳定的 outline/ref/read 边界，本 change 利用核心 CLI 编排这些既有能力，增加一个 human-only interactive workflow，而不改变 adapter 生成 ref 或读取内容的职责。

## What Changes

- 在核心 CLI 中为 `docnav outline <path>` 增加 `--interactive` 模式。
- `--interactive` 模式先执行普通 outline 流程，向用户展示可选择的 outline entries，并允许选择一个或多个条目。
- 用户确认选择后，核心 CLI 按选中的 refs 调用既有 read 流程，避免用户手动复制 ref。
- 交互模式只面向 TTY 人类用户；非 TTY、机器可读输出模式和不支持交互的环境必须有明确行为。
- 第一版交互能力聚焦多选和读取，库选型优先考虑成熟的 Rust prompt/TUI 库；完整树形 TUI、实时预览 pane、复杂快捷键和持久布局属于后续扩展。
- 非目标：不改变 adapter protocol、ref 格式、outline entry 语义、read result 语义或机器可读 JSON 输出契约。

## Capabilities

### New Capabilities

- 无。

### Modified Capabilities

- `core-cli`: 增加核心 CLI 对 `docnav outline <path> --interactive` 的 human-only workflow 行为要求。

## Impact

- 影响 `docnav` 核心 CLI 参数面：新增 `outline --interactive`。
- 影响核心 CLI 执行编排：interactive 模式需要复用 outline outcome，并对选中 refs 顺序执行 read。
- 可能新增 Rust 终端交互依赖。候选方向包括 prompt-style 多选库（如 `inquire`、`dialoguer`）或完整 TUI 框架（如 `ratatui` + terminal backend）。
- 不影响格式 adapter、adapter protocol schema、readable/protocol JSON shape、ref contract 或 adapter 直接 CLI。
- 验证需要覆盖参数互斥、非 TTY 行为、用户取消行为、选择结果到 read 调用的映射，以及不会改变普通 `outline`、`read` 和 JSON 输出模式。
