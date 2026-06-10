---
name: incremental-implementation
description: "以小步增量交付变更；适用于 multi-file change、vertical slice workflows、refactoring、task breakdowns、agent handoffs、verification，或任何无法一步安全完成的工作。"
---

# 增量实现

用 thin vertical slice 推进：每次完成一个最小可验证结果，立即 test/verify，再扩展下一片。目标是让系统在每个 increment 后都保持可运行、可解释、可回滚。

## 读取策略

默认只读本文件。按任务需要加载一层 reference：

1. 需要 slicing examples、feature flag、rollback 或 Docnav vertical slice 示例时，读 [slicing-patterns.md](references/slicing-patterns.md)。
2. 需要写 agent handoff、多人协作边界、red flags 或 scope note 时，读 [agent-collaboration.md](references/agent-collaboration.md)。

## 最小流程

1. **定义 slice**：写清本次唯一目标、可观察结果、涉及边界和验证命令。
2. **确认现状**：查看相关代码、测试和未提交变更；保留用户或其他 worker 的无关改动。
3. **实现一件事**：只改完成本 slice 所需的文件；发现范围外问题时记录，不顺手修。
4. **添加 guard**：用测试、fixture、schema check、smoke 或 manual replay 证明本 slice。
5. **验证结果**：运行与 touched boundary 匹配的最窄命令；代码再次变化后再重跑相关命令。
6. **沉淀证据**：记录变更、验证、残余风险和下一 slice。

## Slice Checklist

每个 slice 开始前确认：

- 目标能用一句话描述，并且只包含一个逻辑变化。
- 结果可通过 test、CLI output、adapter response、MCP tool result 或 UI 行为观察。
- 未完成功能有 feature flag、safe default、compat path 或 additive rollout。
- rollback 路径清楚：最好能独立 revert，或有 migration rollback / compatibility guard。
- 相关验证命令明确，且成本与风险匹配。

每个 slice 完成后确认：

- Build/test 状态没有比开始时更差。
- 新旧行为的期望差异已由 test、fixture 或手动 replay 证明。
- `git diff` 只包含本 slice 范围内的改动。
- 下一步是扩展下一片，而不是重做已验证部分。

## Scope Discipline

把 scope 写成可执行边界：

- **Owned files**：本 slice 会编辑的文件或目录。
- **Read-only context**：只读取以理解行为的文件。
- **Out of scope**：观察到但本轮不处理的改进点。
- **Shared contracts**：protocol、schema、CLI output、adapter contract、MCP mapping 等需要同步验证的边界。

当遇到不相关问题时，把它放入最终总结或 handoff note。只有它阻塞当前 slice 时，才把它升级为新的 slice。

## Docnav 定制切片

Docnav 变更优先按拥有边界切片：

- **Markdown adapter**：先证明 `info`、`outline`、`read`、`find` 或 `invoke` 的单个 behavior。
- **Core CLI**：聚焦 format detection、adapter routing、default limits、output/error mapping。
- **Protocol/schema/examples**：先改 canonical contract，再同步 fixture、example 和 validation。
- **MCP bridge**：只映射到 `docnav` 行为，验证 stdio/JSON tool args 与 result wrapping。
- **Windows path**：保留 drive letter、backslash、spaces、quotes 和 cwd-relative form 作为测试输入。

跨 Rust、Node、schema、docs 或 generated fixtures 时，把每个边界做成独立 slice，并在集成 slice 里跑 workspace verification。

## Verification

按 touched boundary 选择最窄验证：

```bash
cargo test -p docnav-markdown --test adapter -- exact_case_name
cargo test -p docnav-markdown --test cli -- exact_case_name
cargo test -p docnav -- exact_case_name
pnpm run smoke:docnav-markdown
pnpm run smoke:docnav-core
pnpm run verify:docnav-workspace
```

Markdown navigation 行为要手动 replay 关键 path：

```bash
target/debug/docnav-markdown.exe outline path/to/file.md --output protocol-json
target/debug/docnav-markdown.exe read path/to/file.md --ref "L1:Heading" --output protocol-json
```

涉及 raw/readable output 时，分别检查 `text`、`readable-json` 和 `protocol-json`；涉及 adapter `invoke` 时，保存并重放 stdin JSON envelope。

## 完成输出

交付时报告：

- 本次 slices 和 changed files。
- 每个 slice 的 guard / verification。
- 未处理但已记录的 out-of-scope items。
- 仍需下一 slice 处理的风险或阻塞。
