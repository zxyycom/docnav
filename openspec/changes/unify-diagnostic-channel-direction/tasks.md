本 tasks 记录统一 diagnostic channel 的未来推进路径；当前内容只是 `openspec/changes/unify-diagnostic-channel-direction/` 下的未审核临时文档，不改变现有主规范、schema、示例或实现行为。

## 1. 审计门禁

- [ ] 1.1 阻塞级审计：确认 proposal、design、specs 和 tasks 都围绕“内部统一 diagnostic handoff，外部先保持兼容”这一核心句展开；审计未完成前不得执行任何实现任务。
- [ ] 1.2 确认本 change 只修改 `openspec/changes/unify-diagnostic-channel-direction/` 下的未审核临时 artifacts，没有修改现有主规范、schema、示例或代码。
- [ ] 1.3 确认 `docnav-contracts` 是本 change 的正确 capability ID，且没有创建与 `core-cli`、`adapter-protocol`、`readable-view-output` 重复的总括 capability。
- [ ] 1.4 审计当前行为证据：列出 `StableError`、`Warning`、`StandardParameterDiagnostic`、adapter candidate/config source warning 和直接 stderr 诊断的现有 owner、输出通道和测试覆盖。
- [ ] 1.5 决策并记录兼容策略：第一阶段保持 `protocol-json`、manifest、probe stdout 纯净；若要改变 protocol/readable schema，必须拆出显式 breaking contract change。
- [ ] 1.6 决策 `diagnostic_only`：补齐 Rust warning/effect 与测试，或从 readable schema 目标中移除该预留值。

## 2. 诊断模型设计

- [ ] 2.1 定义最小 `DiagnosticEvent` 字段：severity、stable id/code、reason/message、effect、details、source/surface hint 和 fatal/recoverable 语义。
- [ ] 2.2 定义 `DiagnosticBag` 或等价 collection API，支持追加、合并、只读检查、最终 flush 后仍可测试断言。
- [ ] 2.3 明确 `StableError` 与 fatal diagnostic 的关系，避免 stable error details、guidance 或 exit-code category 被重新解释。
- [ ] 2.4 明确 `Warning` 与 recoverable diagnostic 的关系，提供从现有 stable warning envelope 到 event 的兼容转换或投影。
- [ ] 2.5 为 standard parameter validation/source-skipped/ignored argv/adapter candidate failure 设计 family-specific details。

## 3. 兼容迁移实现

- [ ] 3.1 先在 `docnav-diagnostics` 或选定共享层实现 event/bag，保持依赖方向不引入 core 或 adapter SDK 反向依赖。
- [ ] 3.2 迁移 core document operation，使 parse/runtime/output 能携带 result 或 stable error 加 accumulated diagnostics。
- [ ] 3.3 迁移 `docnav-output`，由统一 diagnostics 决定 readable warning 注入和 protocol-json stderr warning flush。
- [ ] 3.4 迁移 adapter direct document operation，使 readable/document protocol 输出与 core 共享同一 diagnostics handoff。
- [ ] 3.5 分阶段收口 adapter SDK 直接 stderr 旁路：direct CLI input error、manifest/probe warning、invoke decode diagnostic 和 output write failure。
- [ ] 3.6 迁移 `docnav-standard-parameters` diagnostics handoff，使 validation failure 和 source-skipped warning 通过统一 event collection 交给 caller。

## 4. 验证

- [ ] 4.1 添加 unit tests，证明 recoverable diagnostics 在跨 parser/runtime/output 边界后仍可取出且不会阻断有效 operation。
- [ ] 4.2 添加 unit tests，证明 fatal error 仍映射为现有 `StableError` code/details/guidance 和 exit code。
- [ ] 4.3 添加 output tests，证明 `protocol-json` stdout 不新增 `warnings` 或 `diagnostics` 字段，warning 仍按兼容策略写 stderr。
- [ ] 4.4 添加 adapter SDK tests，证明 manifest/probe/invoke stdout shape 不变，诊断从 event flush 到 stderr。
- [ ] 4.5 添加 readable tests，证明 `readable-view` 和 `readable-json` 继续使用 stable warning envelope，并覆盖 `diagnostic_only` 的最终决策。
- [ ] 4.6 若实现触及 protocol、schema、examples、output contract 或多个 crate，运行 `bun run verify:docnav-workspace`；否则记录 targeted tests 和未跑全量验证的理由。
