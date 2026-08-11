本 tasks 已收敛为 core service 性能探索；不得实现 adapter service loop、invoke fallback 或 external adapter implementation source。

## 1. Audit Boundary

- [ ] 1.1 确认 service mode 只缓存 core-owned state，不提供 adapter implementation source。
- [ ] 1.2 确认 static registry adapter layer 是 service 与非 service 路径共享的唯一默认 adapter source。
- [ ] 1.3 确认 internal service protocol 不进入 public `docnav-protocol` schema。

## 2. Performance Scope

- [ ] 2.1 定义 startup cost、config/project context loading 和 registry metadata caching 的目标指标。
- [ ] 2.2 设计缓存失效规则，确保 document content、ref 和 parser semantics 仍由 adapter owner 维护。
- [ ] 2.3 设计 doctor/status 输出，避免污染 document success stdout。

## 3. Validation

- [ ] 3.1 增加 service disabled/enabled 等价性测试，证明 protocol/readable success payload 不变。
- [ ] 3.2 增加 benchmark 或 smoke，衡量 cold/hot core startup path。
