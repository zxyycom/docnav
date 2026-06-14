本 change 只在 `openspec/changes/plan-code-quality-observability/` 下记录“先看到代码质量指标，再考虑报告或门禁”的未审核计划，不改变现有实现、主规范或验证链路。

## Why

当前仓库已有格式化、Clippy、schema 和 smoke 验证，但缺少面向代码体量和结构复杂度的可见性。需要先建立一个非阻断的代码质量观测计划，让维护者看到文件行数、符号密度、函数复杂度和后续趋势，再决定是否发展为自动报告或质量门禁。

## What Changes

- 规划一个非阻断的代码质量观测能力，先生成快照和报告，不因指标结果失败。
- 指标首期聚焦文件行数、文件符号数量、函数行数、函数参数数量和圈复杂度。
- 报告首期同时提供机器可读 JSON 和人类可读 Markdown summary，便于后续接入自动报告。
- 明确指标结果只作为定位和排序信号，不在本 change 中定义硬阈值、阻断策略或重构要求。
- 非目标：本 change 不修改现有 Rust/JavaScript 实现，不把质量指标接入 `verify:docnav-workspace` 的阻断链路，不要求现在选择最终第三方度量工具。

## Capabilities

### New Capabilities

- `code-quality-observability`: 定义 Docnav 仓库如何采集、保存和展示非阻断代码质量指标快照。

### Modified Capabilities

- 无。

## Impact

- 未来可能影响 `scripts/` 下的验证或报告脚本、`package.json` scripts、CI artifact 上传策略，以及本地生成的 `target/docnav-quality/` 报告目录。
- 当前影响仅限本 change 目录下的未审核临时 OpenSpec artifacts；不影响 `docnav` CLI、adapter、MCP、schema、examples、主 specs 或 workspace 验证行为。
