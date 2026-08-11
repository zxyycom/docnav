# Tasks

本清单保留 interactive outline 的完整交付顺序；`1.1` 是产品恢复与 Current rebase 的硬门禁，关闭前不得执行 `1.2` 至 `1.8`；`1.3` 和 `1.4` 关闭前不得修改 production。

## Readiness

- [x] 0.1 确认 proposal、design 和 tasks 都以 TTY outline 多选后读取 refs 为同一交付目标，并保留产品延后事实。
- [x] 0.2 确认该 workflow 由 core CLI 拥有，不修改 adapter protocol、ref、operation result、schema 或 machine batch contract。
- [x] 0.3 确认依赖、第一版人类 UX、非 TTY、machine output、取消、空状态、选择顺序和跨平台验证均有明确任务，且依赖门禁与产品 UX 门禁使用不同 owner。

## Implementation

- [ ] 1.1 取得明确的产品恢复确认，并从届时 CLI、output、ref、adapter lifecycle、testing 和 release owner 重新基线；同步本计划后再继续。
- [ ] 1.2 按项目测试流程恢复届时完整当前树与 Case 基线，确认本 design 是实施期间唯一承载 change-local Target 的载体，并登记 CLI/navigation/output/testing/release 的预期 owner delta；此时不把 Target 写成 Current。
- [ ] 1.3 实现子代理按当前官方资料比较 prompt-style 多选候选，以最小 spike 确认 features、维护状态、Windows 终端、取消和非 TTY 行为；由指定 architecture/dependency owner 审阅证据并批准、记录依赖选择。
- [ ] 1.4 实现子代理准备多个 read 的第一版 human UX 候选与取舍证据，由用户或指定 CLI/product owner 明确批准后才能勾选；若不采用推荐的顺序 readable rendering，先更新 proposal/design 和验收条件。
- [ ] 1.5 在 core CLI outline 命令增加 `--interactive`，实现 output-mode 互斥和 TTY 门禁，不把参数传给 adapter。
- [ ] 1.6 建立可测试选择 seam，把 outline display/ref 映射成 options，并按 UI 返回的稳定顺序将 opaque refs 交给既有 read path，同时保持 path、adapter、page、limit 与错误语义。
- [ ] 1.7 实现取消、空选择和空 outline 的不读取路径，并复用既有 readable renderer 输出一个或多个 read results。
- [ ] 1.8 完成 CLI help、语义 Case、必要的用户演示 fixture 和 package smoke 准备，不修改 protocol/schema/examples，也不在本任务中同步稳定 owner 为 Current。

## Verification

- [ ] 2.1 覆盖参数互斥、非 TTY、entry-to-ref、选择顺序、取消、空选择和空 outline，断言失败路径不启动 UI、无读取路径不调用 read。
- [ ] 2.2 在支持终端与 Windows package 上完成 representative smoke，并运行范围匹配的 Rust format、tests 和 lint，形成 implementation/behavior evidence。
- [ ] 2.3 在 `2.1` 与 `2.2` 的实现和行为证据通过后，把 design 登记且已成立的 CLI/navigation/output/testing/release delta 同步为 Current。
- [ ] 2.4 对同步后的 design、稳定 owner、help、Case、package smoke 与实现证据做最终一致性验证；运行 docs/schema unchanged checks 和 `bun run verify:docnav-workspace`，确认普通 outline、protocol-json 和 adapter direct CLI 没有回归。
- [ ] 2.5 审查局部 diff，确认没有 batch protocol、full-screen TUI、adapter UI input、ref 重构造或无关 output abstraction。
