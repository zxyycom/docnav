本任务清单只交付当前树驱动的测试评估体系。`audit.md` 记录当前设计与实现边界，
`verification.md` 记录本 change 的实际验收；旧 case 的逐项去向和恢复流程不属于
交付物。

## 1. 当前契约与工具边界

- [x] 1.1 核对 proposal、design、delta spec 与稳定测试文档的目标、术语、owner
  和非目标，确认没有未回答开放问题。
- [x] 1.2 固定 supported runner profile 的 Cargo targets、Bun surfaces、smoke
  roots、list/report 命令与 selector 语义。
- [x] 1.3 接入项目级 `ast-grep` skill，精确锁定开发期 `@ast-grep/cli` 与
  lockfile，并验证 required check 可离线运行。
- [x] 1.4 证明 external ast-grep 只由项目 wrapper 调用，且不进入 canonical
  release file set 或产品运行时。
- [x] 1.5 建立 Rust、Bun 与 smoke 的正例、近似反例和 unsupported rule tests。

## 2. 原生入口发现与一一闭合

- [x] 2.1 定义 `NativeTestEntry`、确定性 `entryKey`、规范化 source range 与
  source fingerprint。
- [x] 2.2 实现 Rust 静态候选与 Cargo runtime list adapter。
- [x] 2.3 实现 Bun 静态候选与 Bun runner report adapter。
- [x] 2.4 实现 smoke leaf declaration 与 runtime task adapter。
- [x] 2.5 实现 `static-only`、`runtime-only`、`duplicate-entry` 和
  `unsupported-entry-shape` 的双向闭合诊断。
- [x] 2.6 生成完整当前树 inventory，并实现 `missing-case`、`orphan-case`、
  重复与 stale revision 检查。
- [x] 2.7 为 profile、静态、runner、inventory、Claim 与 index 失败保留不同
  origin、退出状态和机器结果。

## 3. Claim、查询与 AI 评估

- [x] 3.1 定义 Evidence Claim 与受控 topic schema，要求精确 owner、稳定语义、
  可观察结果和当前 Entry 支持。
- [x] 3.2 实现未知 owner/Entry/topic、空证据、非法布局、模板复述和
  `claim-stale` 校验，并允许 Entry 没有 Claim。
- [x] 3.3 从 inventory、topic 与 Claims 生成可删除重建的 query index 和双向关联。
- [x] 3.4 实现有界 `topics`、`list`、`show` 查询与不写回的内存投影。
- [x] 3.5 实现显式 baseline 的新增、删除、rename candidate 和
  `implementation-changed` 报告。
- [x] 3.6 在 `test-evidence-review` skill 中固化 owner、观察信号、可靠性、证据
  独立性和信息增量审查。
- [x] 3.7 只按当前语义命名 Claim，删除复述测试名的 Claim，并从 topic 表删除所有
  未使用分类。

## 4. 集成与验收

- [x] 4.1 将 `validate:docs -- cases` 和 workspace verifier 接到项目 wrapper。
- [x] 4.2 同步 `docs/navigation.md`、`docs/testing.md`、
  `docs/testing/case-maintenance.md`、覆盖材料、工具文档、AGENTS 和相关 active
  changes。
- [x] 4.3 运行 ast-grep rule tests、discovery/catalog tests、schema 与 Claim
  查询测试。
- [x] 4.4 重建当前 inventory/index，并运行严格 test-evidence 与 docs validation。
- [x] 4.5 运行 TypeScript typecheck/lint、skill validation 和严格 OpenSpec
  validation。
- [x] 4.6 运行 canonical release file-set 检查与范围匹配的 workspace verification，
  把结果和未消除 warning 写入 `verification.md`。
- [x] 4.7 搜索稳定文档、代码、skill 与 active changes，确认当前链路不再依赖旧
  版本编号、旧 Claim ID、迁移映射或双读路径，并用局部 diff 核对范围。
