本 tasks 只把 `plan-code-quality-observability` 拆成未来可执行步骤；当前仍是未审核临时文档，阻塞级审计完成前不得执行任何实现任务。

## 1. 阻塞级审计

- [ ] 1.1 审计 proposal、design、specs 和 tasks 是否都围绕“先生成非阻断代码质量指标快照和报告，后续再评估自动报告或门禁”这一核心目标。
- [ ] 1.2 审计 capability ID 是否正确使用新的 `code-quality-observability`，且没有创建一次性、实现型或同义 capability。
- [ ] 1.3 审计当前 change 是否只包含 `openspec/changes/plan-code-quality-observability/` 下的未审核临时 artifacts，且没有修改现有 specs、docs、schemas、examples 或实现代码。
- [ ] 1.4 审计是否已明确“本 change 当前仅为计划，不影响现行验证链路、CLI 行为、adapter 行为、MCP 行为或 OpenSpec 主规范”。
- [ ] 1.5 在 1.1-1.4 全部完成前，不得执行任何实现任务、依赖引入任务、CI 接入任务或主规范归档任务。

## 2. 未来指标模型

- [ ] 2.1 定义统一 metrics JSON schema 或等价结构，包含扫描元数据、文件指标、函数指标和聚合指标。
- [ ] 2.2 定义规范化路径、语言、代码区域、文件行数、文件符号数量、函数行数、参数数量和圈复杂度字段。
- [ ] 2.3 定义缺失指标的表达方式，避免不同工具不支持某个字段时伪造数值。
- [ ] 2.4 定义 Markdown summary 的默认栏目、排序规则和 top N 数量。

## 3. 未来采集实现

- [ ] 3.1 评估 `lizard`、`rust-code-analysis`、CodeGraph 和仓库自定义脚本对 Rust/JavaScript 指标的覆盖。
- [ ] 3.2 实现仓库脚本封装选定度量工具，并统一输出路径、字段、排序和错误诊断。
- [ ] 3.3 默认扫描 Rust crates 和仓库 JavaScript 脚本，并排除 `.git`、`target`、`node_modules`、`.venv`、`dist`、`build` 和缓存目录。
- [ ] 3.4 生成 `target/docnav-quality/metrics.json` 和 `target/docnav-quality/report.md` 或等价 artifact 路径。
- [ ] 3.5 确认指标数值不会使命令失败；命令失败仅用于工具不可运行、输入不可读或输出不可写等执行错误。

## 4. 未来报告与验证

- [ ] 4.1 为报告生成逻辑添加单元或 fixture 测试，覆盖排序、分组、缺失指标和路径过滤。
- [ ] 4.2 添加本地脚本入口，例如 `pnpm quality:scan` 或等价命令，并在文档中标注其非阻断性质。
- [ ] 4.3 在 CI 中先以 artifact 或日志形式发布报告，不接入阻断式质量门禁。
- [ ] 4.4 运行与脚本、Rust/JavaScript 指标采集和 workspace 范围匹配的验证命令。
- [ ] 4.5 积累报告样本后，另行评估是否需要 baseline、趋势比较、PR 评论或门禁策略。
