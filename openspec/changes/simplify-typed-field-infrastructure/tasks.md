## 1. Blocking Audit

核心句：先完成 artifact 与消费者审计，再实施一个可独立验证和回滚的优化 slice。当前 tasks 是本 change 目录内的未审核临时计划。

- [ ] 1.1 阻塞审计：确认 proposal、design、specs 和 tasks 围绕核心句；capability ID 使用现有 owner；change 只包含当前目录的临时 artifacts；Open Questions 已解决。审计完成前不得执行任何实现任务。
- [ ] 1.2 核对维护报告、生产消费者和 `derive-document-cli-options-from-fields` 的 supersede 关系，只选择一个首个 slice；在 `design.md` 记录保护行为、owner tests、downstream 和回滚边界。

## 2. Slice Baseline

- [ ] 2.1 运行并记录所选 slice 的 focused baseline，覆盖直接 owner 和受影响 downstream。
- [ ] 2.2 补充最小 characterization 或 contract test，为保护行为提供修改前证明。

## 3. First Optimization Slice

- [ ] 3.1 只实施已审计的首个 slice，保持未入选候选不变。
- [ ] 3.2 同步该 slice 直接拥有的 tests、README/docs 和 dependency metadata，并复核消费者清单。

## 4. Verification and Checkpoint

- [ ] 4.1 运行 focused tests、格式/静态检查、`bun run verify:docnav-workspace` 与 `git diff --check`，确认保护边界不变。
- [ ] 4.2 记录 slice 结果与维护面变化；关闭 change，或先更新全部 artifacts 并只追加一个下一 slice。
