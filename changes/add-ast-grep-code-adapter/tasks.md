# Tasks

本清单保留完整交付顺序；`1.1` 是产品排序与 Current 重新基线的硬门禁，关闭前不得执行 `1.2` 至 `1.9` 或修改 production。

## Readiness

- [x] 0.1 确认 proposal、design 和 tasks 都以 linked 多语言 code adapter 为同一交付目标，并保留产品延后事实。
- [x] 0.2 确认设计不要求修改现有 adapter/ref/protocol 基础契约，也不引入外部 executable、caller rules 或通用 parser abstraction。
- [x] 0.3 确认 implementation 与 verification 任务覆盖 format、private model、ref、operations、registry、owner、Case 和 release package。

## Implementation

- [ ] 1.1 取得明确的产品恢复确认，并从届时 Current adapter、routing、ref、output、testing 和 release owner 重新基线；同步本计划后再开始后续任务。
- [ ] 1.2 按项目测试流程证明完整当前树的实体/Case 映射闭合，确认本 design 是实施期间唯一承载 change-local Target 的载体，并登记 code adapter、architecture/adapter/routing/ref/protocol/output/examples/testing/release 的预期 owner delta；此时不把 Target 写成 Current。
- [ ] 1.3 锁定兼容的 ast-grep crates，关闭全语言默认 features，完成 license、toolchain、feature closure 和 binary-size 基线检查。
- [ ] 1.4 新增 `docnav-code` crate、一个 definition、五个 format mapping、按语言过滤的内置 outline rules 和 adapter-private `CodeSymbol` 转换；覆盖 imports、items、members、Unicode 与可恢复的不完整语法，不增加 shared engine abstraction。
- [ ] 1.5 实现确定性 outline、public entry mapping、file fallback 和 `code:v1` ref formatter/parser；覆盖排序、去重、分页、Unicode boundary、stale digest 与稳定错误分类。
- [ ] 1.6 实现原文 read、literal symbol find 和 stable info；覆盖完整区域 cost、Unicode-safe pagination、pattern-like literal、match-to-read、symbol-free 与 empty source。
- [ ] 1.7 把 definition 加入 static registry，增加 automatic/explicit selection、invocation logging、protocol/readable 和 operation integration tests，且不依赖固定 registry 数量或顺序。
- [ ] 1.8 增加 code fixtures、test-local protocol/readable expected-output fixtures、package-local format 演示、语义 Case 和 coverage mapping，作为实现与行为证明材料；不在本任务中修改稳定 examples 或新增、同步稳定 owner 为 Current。
- [ ] 1.9 更新 canonical package smoke，在 Linux/Windows 支持 target 证明五种 format round trip 且无需外部 ast-grep executable。

## Verification

- [ ] 2.1 运行范围匹配的 Rust format、unit/integration tests、Clippy、schema/example 和文档验证，修复全部失败。
- [ ] 2.2 运行 `bun run verify:docnav-workspace` 以及 required release/package checks，保存 feature closure、binary delta 和 package 行为证据。
- [ ] 2.3 在 `2.1` 与 `2.2` 的实现和行为证据通过后，新增 code adapter 稳定 owner，并把 design 登记且已成立的 architecture/adapter/routing/ref/protocol/output/examples/testing/release delta 同步为 Current。
- [ ] 2.4 对同步后的 design、稳定 owner、实现、Case/examples 和 release/package 证据做最终一致性验证；重新运行 docs/schema/example 与 `bun run verify:docnav-workspace`，再审查局部 diff 与 public surface，确认没有 AST internals、caller rule input、外部 executable path、无关 abstraction 或其它 adapter 回归。
