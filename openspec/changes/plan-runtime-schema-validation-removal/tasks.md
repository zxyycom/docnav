本 tasks 仅记录未来评估运行时 JSON Schema 校验迁移的执行入口；它只在 `openspec/changes/plan-runtime-schema-validation-removal/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## 1. 阻塞级审计

- [ ] 1.1 审计 proposal、design、specs 和 tasks 是否都围绕“未来评估从 release runtime 移除通用 JSON Schema validator，但保留协议契约校验”这一核心目标。
- [ ] 1.2 审计 capability ID 是否正确复用 `adapter-protocol`，且没有创建一次性或同义 capability。
- [ ] 1.3 审计当前 change 是否只包含 `openspec/changes/plan-runtime-schema-validation-removal/` 下的未审核临时 artifacts，且没有修改现有 specs、docs、schemas、examples 或实现代码。
- [ ] 1.4 审计是否已明确“本 change 当前仅为计划，不影响现行规范、运行时行为或 release 流程”。
- [ ] 1.5 在 1.1-1.4 全部完成前，不得执行任何实现任务、依赖迁移任务或主规范归档任务。

## 2. 未来校验覆盖盘点

- [ ] 2.1 盘点 protocol、manifest、probe 和 readable-output schemas 使用的全部 JSON Schema keyword。
- [ ] 2.2 建立 keyword 到 Rust typed validation、semantic validation 或 CI schema validation 的覆盖表。
- [ ] 2.3 识别当前 schema 中需要特别处理的 `$ref`、`oneOf`、`allOf`、`if/then` 和 `pattern` 用法。
- [ ] 2.4 列出迁移后仍必须 fail-closed 的正例和负例 fixture。

## 3. 未来运行时迁移

- [ ] 3.1 为 protocol request/response、manifest 和 probe 补齐或确认 Rust 类型层面的字段、枚举、版本常量和范围约束。
- [ ] 3.2 为 operation/result 绑定、分页语义、manifest/probe 语义和错误映射补齐显式 semantic validation。
- [ ] 3.3 将 release runtime 对通用 JSON Schema validator 的依赖迁移到 dev/test/CI 边界，或替换为等价外部校验流程。

## 4. 未来验证与验收

- [ ] 4.1 增加负例测试，覆盖未知字段、缺失必需字段、类型错误、非法版本常量、非法 operation/result 组合、非法分页字段和非法 manifest/probe payload。
- [ ] 4.2 保留 CI schema 编译、examples 校验和 fixture 校验，证明公开 schema 与实现输出没有漂移。
- [ ] 4.3 对比迁移前后稳定错误分类、失败阶段和字段定位能力。
- [ ] 4.4 对比迁移前后 release binary 体积，并记录是否达到可接受收益。
- [ ] 4.5 运行与协议、adapter SDK、CLI 和 workspace 范围匹配的验证命令。
