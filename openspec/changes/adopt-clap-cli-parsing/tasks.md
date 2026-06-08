**一句话核心：本任务清单把 `clap` CLI 解析迁移拆成审计门禁、规范同步、实现迁移、测试验证和收尾验收。当前 change 只在 `openspec/changes/adopt-clap-cli-parsing/` 下形成未审核临时文档，不影响现有其它文档或主规范。**

## 1. 阻塞级审计门禁

- [ ] 1.1 审计 proposal、design、specs 和 tasks 是否都围绕“使用 `clap` 作为 Rust CLI 参数解析基础，并将直接 CLI 容错收敛为未知输入不阻断成功路径”这一核心句；审计未完成前不得执行任何实现任务。
- [ ] 1.2 确认当前 change 只包含 `openspec/changes/adopt-clap-cli-parsing/` 下的未审核临时 artifacts，且未修改主规范、代码、schema、示例或其它 change。
- [ ] 1.3 确认 delta spec 没有改变 protocol envelope、manifest/probe schema、readable schema、ref 语义或 adapter 格式解析所有权。

## 2. 规范和文档同步

- [ ] 2.1 在完成 1.1-1.3 后，更新 `docs/cli.md`，将直接 CLI 兼容规则改为 `clap` 优先、未知输入不阻断有效成功路径、必要参数错误仍失败的规则。
- [ ] 2.2 更新 `docs/adapter-contract.md`，说明 adapter direct CLI 可以使用 `clap` 完成宽松 argv 解析，但 adapter `invoke` JSON 仍严格校验。
- [ ] 2.3 更新 `docs/testing.md`，把精确 ignored token warning 断言替换为成功路径、必要失败、help 可用和 schema 边界验证。
- [ ] 2.4 使用局部 diff 检查文档改动只覆盖 CLI 参数解析、warning 语义和测试策略相关范围。

## 3. 依赖和解析结构

- [ ] 3.1 在完成 1.1-1.3 后，为相关 Rust crate 添加 `clap` 依赖，并只启用当前实现需要的 feature。
- [ ] 3.2 为 adapter direct CLI 定义 `clap` 命令结构，覆盖 `manifest`、`probe`、`invoke`、`outline`、`read`、`find` 和 `info`。
- [ ] 3.3 将固定参数映射为类型化字段，包括 path、ref、query、page、limit_chars、output 和 Markdown native `max_heading_level`。
- [ ] 3.4 为未知 argv 和多余 positional 设计宽松处理路径，确保必需语义参数正确时不阻断执行。
- [ ] 3.5 保留 adapter `invoke` stdin JSON 的既有 strict schema 校验路径，不让 `clap` 容错进入 protocol 层。

## 4. 实现迁移

- [ ] 4.1 在完成 1.1-1.3 后，替换或包裹 `crates/docnav-adapter-sdk/src/direct/args.rs` 中的手写 argv parser，使直接 CLI 通过 `clap` 或 `clap` builder 产生类型化选项。
- [ ] 4.2 保持 `run_direct_cli` 的 operation request 构造、输出模式分流、稳定错误映射和 warning 承载边界。
- [ ] 4.3 调整 `docnav-markdown` CLI 入口，确保 Markdown native option 通过新的解析结构进入 `arguments.options`。
- [ ] 4.4 删除或收敛不再需要的精确 ignored token、warning kind 和 token 消费顺序实现分支。
- [ ] 4.5 确认 `--help` 和子命令 help 可以输出可读参数说明，并且不会执行文档导航业务。

## 5. 自动化测试

- [ ] 5.1 在完成 1.1-1.3 后，更新 Rust 单元测试，覆盖 `clap` 解析后的类型化参数、必需参数缺失、已知参数非法和 invoke strict 校验。
- [ ] 5.2 更新 `scripts/docnav-markdown-cli-smoke` 的 CLI argument matrix，移除对精确 ignored token shape、kind 和消费顺序的断言。
- [ ] 5.3 增加宽松 argv 成功路径用例，例如 valid path 加 unknown flag、多余 positional 和有效 `--output readable-json`。
- [ ] 5.4 增加 help 用例，验证 `docnav-markdown --help` 或子命令 help 暴露命令和关键参数。
- [ ] 5.5 保留必要失败用例，覆盖缺 path、缺 ref、缺 query、非法 page、非法 limit_chars、非法 max_heading_level、missing file、invalid ref、non-UTF-8 和 malformed invoke JSON。
- [ ] 5.6 验证 protocol-json、manifest 和 probe stdout 不因 CLI warning 增加非 schema 字段。

## 6. 验证和收尾

- [ ] 6.1 在完成 1.1-1.3 后，运行相关 Rust 格式化和静态检查。
- [ ] 6.2 运行 adapter SDK 与 Markdown adapter 的局部测试。
- [ ] 6.3 运行 `pnpm run smoke:docnav-markdown`，确认黑盒 CLI smoke 通过。
- [ ] 6.4 若跨 Rust、文档、schema、示例或输出层边界，运行 `pnpm run verify:docnav-workspace`。
- [ ] 6.5 使用局部 diff 确认实现、文档和测试改动只覆盖本 change 范围。
- [ ] 6.6 更新任务勾选状态，并在最终说明中记录验证命令、结果和任何未解决风险。
