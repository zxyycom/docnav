---
name: claude-code-review-loop
description: 当 Codex 需要把非平凡编码任务委托给 Claude Code，同时仍由 Codex 负责工具审批、diff review、验证和修正循环时使用。权限敏感或工具需求不确定时优先走 Agent SDK approval bridge；工具范围可预先收窄且不需要交互审批时才用 batch dispatcher。
---

# Claude Code Review Loop

目标：让 Claude Code 做实现，Codex 保留范围控制、权限审批、代码审查和最终验收。

## 路线选择

- 默认走 Agent SDK approval bridge：适合非平凡编码、工具需求不确定、或需要 Codex 逐次审批工具调用的任务。
- 只在工具范围已知且无需交互审批时，使用 batch dispatcher。
- 只有机械小改、最终整合，或用户明确要求 Codex 接手时，Codex 才直接编辑。

## 调用脚本

启动：

```powershell
node "<skill_dir>\runtime\claude-approval-cli.mjs" start `
  --working-directory "<repo>" `
  --prompt "<task prompt>" `
  --permission-mode acceptEdits
```

轮询：

```powershell
node "<skill_dir>\runtime\claude-approval-cli.mjs" status --wait-seconds 30
```

审批或拒绝当前请求：

```powershell
node "<skill_dir>\runtime\claude-approval-cli.mjs" approve --request-id "<request-id>" --reason "<reason>"
node "<skill_dir>\runtime\claude-approval-cli.mjs" deny --request-id "<request-id>" --reason "<reason>" --message "<guidance>"
```

命令失败、需要 `stop`、batch dispatcher、并发 session、runtime setup 或维护脚本时，再读取 [approval-bridge.md](references/approval-bridge.md)。

## 核心流程

1. Codex 先读足够上下文，写短 prompt：目标、范围、相关文件、验收标准、验证命令和边界。
2. Codex 按脚本输出审批当前请求，不预授权后续动作。
3. Codex 审查真实 working-tree diff，拒绝无关 churn、过宽重构、缺失错误处理、验证失败或 prompt 范围外行为。
4. Codex 运行与 diff 匹配的验证。需要修复时，用具体反馈开启下一轮。
5. 连续三轮 review 失败后，停止并询问用户方向。

## 审批原则

- Codex 是常规审批者。只有任务本身需要产品或安全决策，且无法从仓库上下文推导时，才问用户。
- 默认使用 `acceptEdits`；需要 Codex 同时审批文件编辑时，使用 `default`。
- 不使用 `bypassPermissions` 做 Codex-managed approval。
- 不接受 Claude 建议的 permission-rule 更新，不持久化宽泛 allow 规则，也不用 local settings 绕开审批。
- 注意：已被 `.claude/settings.local.json` 或其它加载 settings 允许的操作不会进入 `canUseTool`。
- 拒绝破坏性操作、凭据访问、部署、发布、commit、push 或无关网络访问，除非用户任务明确需要该精确动作且 Codex 已审查范围。
- 对 `AskUserQuestion`，能从任务和仓库上下文安全回答时，用 bridge 的 updated input 回答；确实是产品决策时，拒绝请求并问用户。

## Prompt 要点

prompt 保持短而具体，至少写清：

- 要改什么
- 相关文件或区域
- 验收标准
- 范围边界
- 要遵循的现有模式
- 要运行的验证
- 不 commit、push、deploy，不做无关修改
- 报告改动文件和验证结果
