本 change 在 `openspec/changes/implement-code-quality-observability/` 下实现非阻断代码质量观测能力。实现范围包含仓库质量扫描脚本、配置文件、临时产物目录、CI 汇报入口和对应验证；不改变 `docnav` CLI、adapter、MCP、schema 或 examples 的业务契约。

## Why

当前仓库已有格式化、Clippy、schema 和 smoke 验证，但缺少面向代码体量、语言占比、文件复杂度、函数复杂度和重复代码的可见性。需要实现一个非阻断的代码质量观测入口：Clippy 继续作为 Rust 阻断式质量门禁，Lizard 负责函数级复杂度快照和 warning 来源，scc 负责仓库体量、语言占比、文件级复杂度和趋势报告输入，PMD CPD 负责重复代码检测信号。

## What Changes

- 实现一个仓库封装的非阻断代码质量观测命令，统一消费 Lizard、scc 和 PMD CPD 输出。
- 保持 Clippy 作为 Rust 阻断式质量门禁；Lizard、scc 和 PMD CPD 指标只生成快照、warning 和报告，不因指标值阻断现有验证。
- 新增配置文件，定义扫描范围、排除规则、默认 6 类 code areas、generated files、warning 规则和工具参数。
- 指标首期覆盖仓库体量、语言占比、文件行数、文件级复杂度、函数行数、函数参数数量、函数圈复杂度和重复代码片段。
- Dynamic warning 和 PMD CPD `minimum tokens` 首期按 code area 拆分，默认覆盖 Rust production、Rust tests、Node production scripts、Node validation/smoke scripts、fixtures/examples 和 generated。
- 报告首期写入临时产物目录，提供机器可读 `metrics.json`、人类可读 `report.md`、warning records，以及必要的第三方原始输出归档。
- `metrics.json` 首期记录 current snapshot、同次运行生成的 previous-code baseline snapshot、扫描输入指纹、baseline status、comparison status 和按 code area 的 delta；text-only 变更记录 `input-unchanged`，不产生复杂度或重复代码 annotation。
- 在 CI 中正常产出 artifact、step summary 和非阻断 warning annotation；本 change 不把 Lizard、scc 或 PMD CPD 的指标值接入 `verify:docnav-workspace` 的阻断链路。
- 非目标：本 change 不修改 `docnav` CLI、adapter、MCP、schema 或 examples 的业务行为，不替代 Clippy、测试、schema 验证、smoke 验证或人工 code review。

## Capabilities

### New Capabilities

- `code-quality-observability`: 定义 Docnav 仓库如何用 Clippy、Lizard、scc 和 PMD CPD 分层获得代码质量信号，并如何采集、保存、展示和汇报非阻断指标快照。

### Modified Capabilities

- 无。

## Impact

- 影响 `scripts/` 下的质量扫描和报告脚本、`package.json` scripts、质量观测配置、CI artifact 上传策略、PR warning/summary 汇报策略，以及本地/CI 临时质量报告目录。
- 不影响 `docnav` CLI、adapter、MCP、schema、examples 或主 specs 的业务契约；`verify:docnav-workspace` 可以继续保留现有阻断验证语义。
