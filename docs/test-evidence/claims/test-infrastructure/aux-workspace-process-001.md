# Claim CLAIM-AUX-WORKSPACE-PROCESS-001: Shared process wrapper plain-text environment 稳定

Topic: `test-infrastructure`
Owner ref: `docs/tooling.md#子进程输出环境`

Statement:
- Shared script process wrappers override caller color settings with the repository plain-text child environment.

Observations:
- shared process wrapper 在 sync 和 async child process 入口覆盖 caller-provided color env，统一注入 plain-text output environment。

Supported by:
- `bun|scripts/tools/foundation/test/foundation.test.ts|script foundation > runs child processes with plain text output environment`
