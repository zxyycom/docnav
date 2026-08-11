## 1. Change Audit

- [x] 1.1 审计 proposal、design、spec delta 和 tasks 是否围绕运行期目录分离；确认 capability ID 复用现有 `code-quality-observability` 与 `markdown-navigation`。

## 2. Runtime Path Implementation

- [x] 2.1 将 workspace verifier 日志目录改为 `.log/verify/workspace/`，并将 dev binary env file 移入 `.cache/docnav/verify/`。
- [x] 2.2 将 core smoke 日志目录改为 `.log/smoke/core/`，并将 core smoke 临时工作区移入 `.tmp/docnav/smoke/core/<timestamp>/`。
- [x] 2.3 将 Markdown smoke 日志目录改为 `.log/smoke/markdown/`，并更新终端输出和 verifier 输出过滤期望。
- [x] 2.4 将 quality duplicate-code scan cache 从 `.log/docnav-quality-cache/<scan_cache_version>/` 移入 `.cache/docnav/quality/<scan_cache_version>/`。
- [x] 2.5 确保 quality scan 报告默认继续写入 `artifacts/docnav-quality/`。

## 3. Documentation and Configuration

- [x] 3.1 更新 `.gitignore`、ESLint ignore 和 quality scan exclude dirs，使 `.tmp/` 与 `.cache/` 不进入源码扫描或 lint。
- [x] 3.2 更新 `docs/testing.md` 中 workspace verifier 和 smoke 日志路径说明。
- [x] 3.3 更新相关测试期望、case 文档或 OpenSpec 验证材料，保持路径契约一致。

## 4. Verification

- [x] 4.1 运行 OpenSpec change validation，确认 spec delta 可解析。
- [x] 4.2 运行受影响脚本测试，包括 workspace verifier、smoke harness 和 quality cache tests。
- [x] 4.3 运行 `bun run verify:docnav-workspace:required` 或说明未运行原因。
- [x] 4.4 检查最终 diff 只包含 runtime directory layout change 相关文件。
