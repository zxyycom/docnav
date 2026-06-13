# Approval Bridge 失败和维护补充

只有在入口命令失败、需要 stop、concurrent session、runtime setup，或维护 approval bridge 脚本时读取本文件。

## Runtime

bridge 有独立 Node runtime。缺少 `runtime/node_modules` 时：

```powershell
cd "<skill_dir>\runtime"
pnpm install --frozen-lockfile
```

不要把 Agent SDK 安装到仓库根 workspace。

bridge 会解析并传入系统 `claude` executable 给 SDK；不能静默回退到 SDK bundled binary。保持 `claude` 在 `PATH` 上，或在 `start` 时传入 `--claude-executable "<absolute-path>"`。

bridge 继承当前环境，加载 Claude Code 的 user/project/local settings，并使用 `claude_code` system prompt 和 tool presets。

## 常用命令

启动：

```powershell
node "<skill_dir>\runtime\claude-approval-cli.mjs" start `
  --working-directory "<repo>" `
  --prompt "<task prompt>" `
  --permission-mode auto
```

prompt 含 shell-sensitive 字符时，改用 `--prompt-file "<path>"`。默认输出包含 `session: <session-id>`；需要完整 snapshot、stderr 或进程细节时，加 `--json`。

轮询：

```powershell
node "<skill_dir>\runtime\claude-approval-cli.mjs" status --session-id "<session-id>" --wait-seconds 1800
```

审批或拒绝当前请求：

```powershell
node "<skill_dir>\runtime\claude-approval-cli.mjs" approve `
  --request-id "<request-id>" `
  --reason "<why this exact action is safe and necessary>"

node "<skill_dir>\runtime\claude-approval-cli.mjs" deny `
  --request-id "<request-id>" `
  --reason "<risk or scope violation>" `
  --message "<guidance Claude should follow instead>"
```

需要用户决策时，不要先 deny；保持请求 pending，问用户，拿到决定后再运行 approve 或 deny。

停止：

```powershell
node "<skill_dir>\runtime\claude-approval-cli.mjs" stop --session-id "<session-id>" --reason "<reason>"
```

CLI 默认输出是精简文本。`--json` 用于调试、并发 session 诊断、stderr、进程细节或机器处理。

## 等待策略

- approval session 在后台运行；用有界 `status --wait-seconds` 轮询，最大 1800 秒。
- CLI 记住最新 session；只有单 session 场景可省略 `--session-id`。
- 并行 session 时，记录每个 `start` 或 `status` 输出里的 `session`；`status` 和 `stop` 必须显式传 `--session-id`。
- `approve` 和 `deny` 不接收 session 路径，只用 `--request-id` 自动定位 pending request 所属 session。
- Claude Code 可能很慢。等待 30 分钟后仍超时，就拆小任务。

## 维护检查

- 修改 runtime code 后，在 `runtime/` 运行 `pnpm run check`。
- 修改输出格式后，同时运行 `pnpm test`。
- 修改 bridge、controller、permissions 或输出后，验证真实 `start/status/approve/deny/stop` 行为。

维护文件时的粗略边界：

- `claude-approval-cli.mjs`：命令解析和输出模式选择。
- `approval-session-controller.mjs`：session 生命周期和审批决策。
- `approval-session-output.mjs`：默认精简输出格式。
- `approval-session-store.mjs`：文件协议和原子写入。
- `claude-approval-bridge.mjs`：Agent SDK `query` 和 `canUseTool`。
