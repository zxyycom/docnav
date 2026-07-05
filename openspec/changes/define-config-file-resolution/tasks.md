## 一句话核心

本任务清单把 user/project config file path resolution change 拆成先审计、再实现、最后验证的步骤；当前 change 只在 `openspec/changes/define-config-file-resolution/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## 文档状态

本 tasks artifact 是未执行清单。阻塞级审计完成前，不得执行任何实现任务。

## 1. 实现前阻塞审计

- [x] 1.1 审计 `proposal.md`、`design.md`、`specs/core-cli/spec.md`、`specs/navigation-input-resolution/spec.md` 和本文件，确认所有内容都围绕“定义 user/project config file path flags 与默认解析顺序”这一核心句。
- [x] 1.2 确认 capability ID 只使用已有 `core-cli` 和 `navigation-input-resolution`，没有创建一次性或同义 capability。
- [x] 1.3 确认当前 change 只包含 `openspec/changes/define-config-file-resolution/` 下的未审核临时 artifacts，没有修改现有主规范、docs、schema、examples 或实现文件。
- [x] 1.4 确认 `design.md` 的 Open Questions 没有未回答问题，尤其是 `DOCNAV_CONFIG_DIR` 目录语义、project context 默认和 explicit missing failure。
- [x] 1.5 审计未完成前不得执行 2.x 及后续任何实现任务。

## 2. Contract Documentation

- [x] 2.1 更新 `docs/cli.md`，记录 `--project-config <path>`、`--user-config <path>` 的支持命令、help/strict input 边界和 config/init/doctor 目标行为。
- [x] 2.2 更新 `docs/navigation-input-resolution.md`，记录 core-supplied descriptor 的 path origin、default missing absence、explicit missing failure 和参数来源优先级不变。
- [x] 2.3 更新 `docs/architecture.md` 中 core 与 navigation handoff 摘要，只引用 owner 文档，不重复完整规则。
- [x] 2.4 更新 `docs/testing.md`、`docs/testing/case-maintenance.md` 或 `docs/testing/cases.md` 中受影响的测试证明目标和 case ledger。

## 3. Core CLI Path Resolution

- [x] 3.1 扩展 CLI parser 和 command model，为 document operations、`config`、`doctor` 和 `init` 增加共享 config path args。
- [x] 3.2 实现 `--project-config` 和 `--user-config` 的 exact file path 解析，确保缺值、未知 flag 和未文档化命令上的 flag 走 strict input diagnostic。
- [x] 3.3 实现 user config path fallback：`--user-config`、`DOCNAV_CONFIG_DIR/docnav.json`、平台用户默认 `.docnav/docnav.json`。
- [x] 3.4 实现 project config path fallback：`--project-config`、当前 project context 的 `.docnav/docnav.json`。
- [x] 3.5 为 project/user config descriptor 增加 source level、resolved path 和 path origin，供 document operation 和 context helpers 复用。

## 4. Command Behavior

- [x] 4.1 更新 document operation handoff，使 core 把 selected project/user config descriptors 传给 `docnav-navigation`。
- [x] 4.2 更新 `config get|set|unset|list`，使读写目标来自 selected config path；`--user` 仍只决定 scope，不决定文件路径。
- [x] 4.3 更新 `config list --path --operation` 的 document context 解析，使 selected project/user config files 参与 context 输出。
- [x] 4.4 更新 `init --project-config`，使它创建或保留 selected project config file，并保持幂等。
- [x] 4.5 更新 `doctor`，使它检查 selected project/user config files 和对应 failure/absence 状态。

## 5. Navigation Source Loading

- [x] 5.1 更新 `docnav-navigation` config source descriptor/loading model，接收并保留 path origin。
- [x] 5.2 实现 default-path missing config source 为 absent，不产生 diagnostic。
- [x] 5.3 实现 explicit-path missing、unreadable、invalid JSON 和顶层非 object 为 blocking config source diagnostic。
- [x] 5.4 确认 config path flag selection 不改变参数值来源优先级，文件内部值仍按 project/user source level 参与合并。
- [x] 5.5 确认 config source diagnostics 包含 source level 和 selected config file path。

## 6. Tests

- [x] 6.1 添加 Rust parser/path resolution tests，覆盖支持命令、strict failure、exact file path 和 fallback order。
- [x] 6.2 添加 core config command tests，覆盖 `config set/list` 对 explicit project/user config files 的读写目标。
- [x] 6.3 添加 navigation config source loading tests，覆盖 default missing absence、explicit missing failure 和 explicit invalid source diagnostics。
- [x] 6.4 更新 core CLI smoke，使只读配置 fixture 可直接通过 config path flag 使用，会写配置先复制到临时文件再传 path。
- [x] 6.5 更新测试 case ledger 和源码 `@case` 标记，证明新增 public contract 有对应验证。

## 7. Verification

- [x] 7.1 运行与改动范围匹配的 Rust unit/integration tests 和 core CLI smoke。
- [x] 7.2 运行 `bun run verify:docnav-workspace` 或更严格的 workspace verification profile，覆盖 CLI、docs、testing ledger 和 cross-boundary behavior。
- [x] 7.3 运行 `openspec validate "define-config-file-resolution" --type change --strict --no-interactive`。
- [x] 7.4 用局部 diff 确认实现只改动本 change 要求的 CLI/config/navigation/docs/tests 范围。
