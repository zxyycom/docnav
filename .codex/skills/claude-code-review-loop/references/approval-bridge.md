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

prompt 含 shell-sensitive 字符时，改用 `--prompt-file "<path>"`。需要完整 snapshot 或 session directory 时，加 `--json`。

轮询：

```powershell
node "<skill_dir>\runtime\claude-approval-cli.mjs" status --wait-seconds 30
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

停止：

```powershell
node "<skill_dir>\runtime\claude-approval-cli.mjs" stop --reason "<reason>"
```

CLI 默认输出是精简文本。`--json` 用于调试、并发 session、完整路径、stderr、进程细节或机器处理。

## 等待策略

- approval session 在后台运行；用有界 `status --wait-seconds` 轮询。
- CLI 记住最新 session；非并发场景可省略 `--session-directory`。
- Claude Code 可能很慢。重试后仍超时，就拆小任务。

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
