# Agent Collaboration

本引用用于要求 agent 或并行 worker 做小步实现、handoff、范围记录和 red flag 判断时。

## Slice Prompt Template

```text
Implement one slice: <single outcome>.

Owned files:
- <paths>

Read-only context:
- <paths>

Out of scope:
- <items>

Guard / verification:
- <commands or manual replay>

Stop and report if:
- <conditions that require a new slice or user decision>
```

把 "done" 定义成可验证结果，例如 "adapter `outline --output protocol-json` includes the expected ref"。

## Handoff Fields

多人协作或长任务交接时使用这些字段：

- **Goal**：当前 slice 的唯一结果。
- **Changed files**：实际编辑文件。
- **Evidence**：通过的 tests、CLI replay、schema checks 或 screenshots。
- **Contracts touched**：protocol、schema、raw/readable output、adapter contract、MCP mapping。
- **Known limits**：本 slice 有意没覆盖的路径。
- **Next slice**：最小后续动作。

## Scope Notes

记录范围外观察时，用可追踪但不打断当前 slice 的格式：

```text
Observed out of scope:
- <file/path>: <issue>. Suggested follow-up: <small next task>.
```

只有该问题阻止当前 slice 验证时，才把它转为 blocker。

## Red Flags

出现这些信号时，收窄 slice：

- 写了约 100 行以上代码还没有 guard 或 replay。
- 一个 diff 同时改 feature、refactor、formatting 和 generated files。
- Build 或 tests 在 slice 之间保持失败，且失败原因没有记录。
- 为一个真实用例创建通用 framework、registry 或 config language。
- 连续运行同一个已通过命令，但代码和输入没有变化。
- 触碰 parallel worker 的目录、用户未要求的文件或只读 reference。

## Recovery Moves

当 slice 变大时：

1. 暂停新编辑，运行当前最窄验证。
2. 用 diff 把变更分组为 behavior、test、fixture、docs、cleanup。
3. 保留能证明当前目标的最小组，其余改成后续 slice 或撤回自己刚做的无关编辑。
4. 在总结中写清仍未处理的后续片。
