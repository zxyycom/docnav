---
name: claude-code-subagent
description: 在长时间、非平凡或多阶段工程任务中，通过启动 Claude Code 子代理承接实现、独立 code review、多角度审查、探索和修正循环，将高噪声执行过程隔离到子上下文中。主代理保留任务范围、关键审批、结果验收和最终决策权，从而减少主上下文被细节、试错和中间状态污染，保持长任务中的判断一致性和执行连续性。
---

# Claude Code Subagent

目标：在长任务中保持主代理上下文一致性。把实现、独立 CR、多角度 CR、探索和修正循环委托给 Claude Code 子代理，让高噪声执行过程发生在隔离子上下文中；主代理只保留范围控制、关键审批、结果验收和最终决策。

## 路线选择

- 默认走 Agent SDK approval bridge：适合长时间、非平凡、多阶段或需要独立视角的工程任务，主代理负责设定范围、审批关键动作、验收子代理结果和最终决策。
- 只有机械小改、最终整合，或用户明确要求主代理直接编辑时才跳过 bridge。

## 调用脚本

启动：

```powershell
node "<skill_dir>\runtime\claude-approval-cli.mjs" start `
  --working-directory "<repo>" `
  --prompt "<task prompt>" `
  --permission-mode auto
```

记录输出中的 `session: <session-id>`；并行 session 时，每个任务单独记录自己的 session id。

轮询：

```powershell
node "<skill_dir>\runtime\claude-approval-cli.mjs" status --session-id "<session-id>" --wait-seconds 1800
```

审批或拒绝当前请求：

```powershell
node "<skill_dir>\runtime\claude-approval-cli.mjs" approve --request-id "<request-id>" --reason "<reason>"
node "<skill_dir>\runtime\claude-approval-cli.mjs" deny --request-id "<request-id>" --reason "<reason>" --message "<guidance>"
```

并行 session 时，`status` 和 `stop` 必须传 `--session-id`；`approve` 和 `deny` 只传 `--request-id`，runtime 会自动定位 pending request 所属 session。

命令失败、需要 `stop`、runtime setup 或维护脚本时，再读取 [approval-bridge.md](references/approval-bridge.md)。

## 核心流程

1. 主代理先读足够上下文，写短 prompt：目标、范围、相关文件、验收标准、验证命令和边界。
2. 工具调用按权限模式自动审批或经主代理审批；不预授权后续动作。
3. 主代理审查真实 working-tree diff，拒绝无关 churn、过宽重构、缺失错误处理、验证失败或 prompt 范围外行为。
4. 主代理运行与 diff 匹配的验证。需要修复时，用具体反馈开启下一轮。
5. 连续三轮 review 失败后，停止并询问用户方向。

## 审批原则

- 常规任务使用 `auto`（默认）：模型分类器自动审批工具调用，主代理集中做最终 diff review 和验收。
- 涉及核心协议、schema、安全敏感代码或跨模块重构时使用 `acceptEdits`：Bash/网络等操作由主代理逐次审批，编辑仍自动通过。
- 需要主代理逐次审批包括编辑在内的所有操作时，使用 `default`。
- 主代理保留最终验收权：无论哪种模式，最终 diff 必须经主代理审查通过。
- 超出主代理判断范围的产品或安全决策，保持当前请求 pending，向用户提问；拿到用户决定后，再由主代理继续 approve 或 deny。
- 不使用 `bypassPermissions` 绕过审批流程。
- 不接受 Claude 建议的 permission-rule 更新，不持久化宽泛 allow 规则，也不用 local settings 绕开审批。
- 注意：已被 `.claude/settings.local.json` 或其它加载 settings 允许的操作不会进入 `canUseTool`。
- 拒绝破坏性操作、凭据访问、部署、发布、commit、push 或无关网络访问，除非用户任务明确需要该精确动作且主代理已审查范围。
- 对 `AskUserQuestion`，能从任务和仓库上下文安全回答时，用 bridge 的 updated input 回答；确实需要用户决策时，保持请求 pending，先问用户，拿到决定后再用 updated input 回答或按用户决定拒绝。
