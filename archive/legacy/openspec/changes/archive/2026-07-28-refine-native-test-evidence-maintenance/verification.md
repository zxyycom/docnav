# Verification

本记录保存本 change 在 2026-07-27 的最终验证证据；命令结果只证明当前
checkout，不代表尚未运行的外部环境。

## Targeted evidence

- `bun test scripts/test-evidence/discovery/smoke-fingerprint.test.ts
  scripts/test-evidence/discovery/bun-files.test.ts
  scripts/test-evidence/catalog.test.ts` 通过 13 个测试，证明目录展开与符号链接边界、
  正向 glob 语义、smoke 可达实现 fingerprint、profile v2、schema 示例和 sync
  自举边界。
- `bun run typecheck:scripts`、`bun run lint:scripts` 与
  `cargo fmt --all -- --check` 通过。
- `cargo test -p docnav-adapter-contracts` 通过，共 8 个测试。
- `CORE-MD-OPTIONS-001` 与新增的 `CORE-MD-OPTIONS-002` selector 分别独立通过；
  完整 workspace verifier 随后通过当前 27 个 core smoke leaf。
- `bun run test:test-evidence` 通过 19 个测试；
  `bun run test:test-evidence-rules` 通过 9 组 ast-grep rule tests。
- `bun run validate:docs` 与
  `openspec validate refine-native-test-evidence-maintenance --strict`
  通过。

## Ledger result

- `sync --write` 从完整当前树重建 inventory/index。
- 随后的 strict check 通过：551 个原生入口，其中 Cargo 393、Bun 131、
  smoke 27；21 个 Claim。
- 31 个入口关联 Claim，520 个入口不使用 Claim；7 个 Claim 关联多个入口。
- 相对 change 前 inventory 的 `changes` 报告已审查新增、删除、
  implementation change 和 rename candidate：18 added、3 removed、
  18 implementation-changed、3 rename candidates。`CORE-MD-OPTIONS-001`
  现在能报告实现变化，`CORE-MD-OPTIONS-002` 作为独立入口新增；未保留旧 Entry
  alias。

## Workspace result

最终 `bun run verify:docnav-workspace` 完成 15 项检查：

- 14 项 passed；
- 0 项 failed；
- quality full check 为 warning。

quality artifact 记录 18 条没有 accepted reason 的 warning，其中 10 条按当前
quality baseline 分类为 changed/regression。它们没有造成验证失败；本 change
新增的 Bun 目录展开与 smoke fingerprint 实现已经按路径解析、source 解析和
可达图归因边界收敛，没有留下新的局部复杂度 warning。剩余观测保存在
`artifacts/docnav-quality/report.md` 和
`artifacts/docnav-quality/warnings-all.ndjson`。
