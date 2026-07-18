本 tasks 列出 `interactive-outline-selection` 的实现前审计门禁和后续工作分解：为 `docnav outline <path>` 增加面向人类的交互式选择与 read 编排；当前 change 只在 `openspec/changes/interactive-outline-selection/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## 1. 阻塞级审计门禁

- [ ] 1.1 审计 proposal、design、specs 和 tasks 是否都围绕“交互式 outline 选择后读取选中 refs，避免手动复制粘贴”这一核心目标；审计未完成前不得执行任何实现任务。
- [ ] 1.2 审计 capability ID 是否正确复用 `core-cli`，且没有把 change name 误作为长期 capability；审计未完成前不得执行任何实现任务。
- [ ] 1.3 审计本 change 是否只包含 `openspec/changes/interactive-outline-selection/` 下的未审核临时 artifacts，且未修改现有主规范、docs、schema、examples 或应用代码；审计未完成前不得执行任何实现任务。
- [ ] 1.4 审计 spec delta 是否只新增 core CLI human-only workflow 要求，且没有改变 adapter protocol、ref contract、readable/protocol JSON shape 或 adapter 直接 CLI；审计未完成前不得执行任何实现任务。

## 2. 依赖与交互方案确认

- [ ] 2.1 基于当前 Rust workspace 和官方文档比较 `inquire`、`dialoguer`、`ratatui` 候选，记录选择理由、版本、feature、Windows 终端行为和依赖影响。
- [ ] 2.2 用最小 spike 验证候选库可以在 Windows PowerShell/终端中完成多选、取消和确认，并能在非 TTY 环境中被可靠拦截。
- [ ] 2.3 确认第一版输出 UX：多选确认后是顺序渲染 read 结果，还是进入临时查看界面；若选择临时查看界面，先更新 design 和 spec 再实现。

## 3. Core CLI 参数与边界

- [ ] 3.1 在 core CLI outline 命令中增加 `--interactive` 参数，并保持普通 `outline` 参数和 help 行为不变。
- [ ] 3.2 实现 `--interactive` 与 `--output protocol-json` 的互斥校验，返回 `INVALID_REQUEST` 且不启动交互 UI；省略 output 或显式 `readable-view` 继续使用人类终端流程。
- [ ] 3.3 实现非 TTY stdin/stdout 环境的 `--interactive` 拒绝路径，返回 `INVALID_REQUEST` 并给出明确诊断。
- [ ] 3.4 确认 `--interactive` 不进入 adapter `invoke` stdin JSON，也不作为 adapter native option 传递。

## 4. Interactive Workflow

- [ ] 4.1 复用现有 outline semantic request、adapter selection、invoke 和 output outcome 逻辑获取 entries。
- [ ] 4.2 将 outline entries 映射成交互选项，保证 display 用于人类展示，ref 作为后续 read 的唯一业务输入。
- [ ] 4.3 实现用户确认选择后按选择顺序执行既有 read 语义，并保留 read 的 path、adapter、page、limit_chars 和错误映射规则。
- [ ] 4.4 实现用户取消路径：不执行 read，并以成功退出或等价的用户取消结果结束。
- [ ] 4.5 实现空 outline 路径：不展示多选界面，不执行 read，并给出人类可读结果。

## 5. Verification

- [ ] 5.1 为参数互斥和非 TTY 拒绝路径增加自动化测试，断言不启动交互 UI、不输出机器可读 JSON 混合内容。
- [ ] 5.2 为 outline entry 到 selected ref 到 read invocation 的映射增加可测试 seam 或等价测试，断言按选择顺序读取。
- [ ] 5.3 为用户取消和空 outline 行为增加测试或可复现 smoke，断言不执行 read。
- [ ] 5.4 运行范围匹配的 Rust tests、format/lint 和 Docnav workspace 验证；若涉及协议、schema、示例或多个包边界，运行 `bun run verify:docnav-workspace`。

## 6. 文档与收尾

- [ ] 6.1 按 `docs/navigation.md` 的 owner 规则同步更新 CLI、输出模式或测试文档中需要长期保留的行为说明。
- [ ] 6.2 确认未修改 adapter protocol、ref contract、readable/protocol JSON schema 和 examples；若发现实现需要修改，先回到 OpenSpec 更新 proposal/design/spec。
- [ ] 6.3 用局部 diff 检查本 change 的实现范围，只包含交互式 outline 相关代码、测试和必要文档。
