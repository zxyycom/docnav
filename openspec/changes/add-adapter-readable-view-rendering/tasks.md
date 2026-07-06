本 tasks 清单拆解 `readable-view` adapter 可选自定义渲染 hook 与 Markdown adapter md-like 输出的实施步骤；当前文档只在 `openspec/changes/add-adapter-readable-view-rendering/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## 1. 阻塞级审计

- [ ] 1.1 审计 proposal、design、specs 和 tasks 是否都围绕“adapter 可选注入 readable-view 自定义文本渲染，Markdown adapter 首期选择 md-like 输出”这一核心目标。
- [ ] 1.2 审计 capability ID 是否只复用现有 `output-contract` 和 `markdown-adapter`，不创建同义新 capability。
- [ ] 1.3 审计当前 change 是否只包含 `openspec/changes/add-adapter-readable-view-rendering/` 下的未审核临时 artifacts，且没有修改主规范、schema、example 或实现代码。
- [ ] 1.4 审计通用 hook 文案是否没有暗示所有 adapter 必须实现 md-like、native-like、省略标记、ref guidance 或 continuation guidance。
- [ ] 1.5 审计 `## Open Questions` 是否没有未回答问题或已收敛歧义；若 generic fallback 形态仍需单独决策，先记录为后续 change 或 implementation note。
- [ ] 1.6 审计未完成前不得执行任何实现任务；只有 1.1-1.5 全部完成后才能开始第 2 组及之后任务。

## 2. Contract 与文档同步

- [ ] 2.1 更新 `docs/output.md`，把 `readable-view` 成功路径描述为 adapter-rendered text 优先、core generic fallback 兜底，并保留 `readable-json` / `protocol-json` 稳定边界。
- [ ] 2.2 更新 `docs/adapter-contract.md` 或对应 owner 文档，定义 adapter readable-view renderer hook 的输入、输出、unsupported、failure fallback 和禁止读取进程边界的规则，并明确通用 hook 不要求 md-like、省略、ref guidance 或 continuation guidance。
- [ ] 2.3 更新 `docs/adapters/markdown.md`，说明 Markdown outline/read/find/info 的 md-like readable-view、省略标记语义、ref 可发现性和 continuation guidance。
- [ ] 2.4 更新受影响 examples、fixtures、goldens 或 schema 索引材料；若某项不适用，在实现记录中说明 owner 和不更新原因。

## 3. Core readable-view orchestration

- [ ] 3.1 定义内部 render context 和 adapter hook return type，表达 `rendered text`、`unsupported` 和 renderer-local failure。
- [ ] 3.2 在 `docnav-output` readable-view 成功路径中优先调用 selected adapter renderer，并确保 `readable-json` 和 `protocol-json` 不调用 presentation hook。
- [ ] 3.3 实现 adapter renderer 缺失、unsupported 或 renderer-local failure 时的 generic fallback，且在 fallback 前不写 partial stdout。
- [ ] 3.4 保留 generic fallback 的稳定失败诊断和 stdout/stderr/exit code 映射；fallback 本身失败时继续返回 readable-view render failure。

## 4. Markdown md-like renderer

- [ ] 4.1 为 Markdown outline 实现 heading-shaped readable-view，保持文档顺序、ref 可发现性，并为 omitted siblings、children 或 body detail 输出显式省略标记。
- [ ] 4.2 为 Markdown read 实现 selected content/page readable-view，保留 Markdown-like content、selected ref、page 前后省略标记和 continuation guidance。
- [ ] 4.3 为 Markdown find 实现 match context readable-view，按文档顺序展示命中上下文、完整 ref、非连续区域省略标记和更多结果 guidance。
- [ ] 4.4 为 Markdown info 实现 Markdown-readable 摘要，表达格式身份、content type 和 operation result 中已有的文档事实。
- [ ] 4.5 统一 Markdown omission marker 和 ref/continuation 文案，确保 markers 明确属于 readable-view projection 而非原文内容。

## 5. Verification

- [ ] 5.1 为 core orchestration 添加测试，覆盖 adapter renderer success、unsupported、renderer-local failure fallback、fallback failure、machine output bypass 和 no partial stdout。
- [ ] 5.2 为 Markdown renderer 添加 operation-level 测试，覆盖 outline、read、find、info、无 heading、分页、大文档、Unicode、CRLF、frontmatter 和代码围栏伪 heading。
- [ ] 5.3 更新 core CLI Markdown smoke/golden，证明 `readable-view` 使用 md-like text，`readable-json` 和 `protocol-json` 继续符合对应 shape。
- [ ] 5.4 运行 OpenSpec 验证：`openspec validate add-adapter-readable-view-rendering --type change --json --strict --no-interactive`。
- [ ] 5.5 运行范围匹配的 Rust、schema/example 和 workspace 验证；跨 core/output/adapter 边界后优先运行 `bun run verify:docnav-workspace`。
