本 tasks 是 `extract-shared-script-toolkits` 的 change-stage checklist；实现前必须先完成脚本提取范围和 owner 审计。

## 1. 阻塞级审计门禁

- [ ] 1.1 审计 proposal、design、spec 和 tasks 是否都围绕“提取 Docnav 脚本中可复用工具内核，形成子仓库或可发布包，并保留 Docnav 专属策略”这一目标。
- [ ] 1.2 审计 capability ID 是否只新增 `shared-script-tooling`，没有把 `repository-quality-observability` 或 `release-artifacts` 当作共享工具 owner。
- [ ] 1.3 审计 proposal 阶段改动是否只位于 `openspec/changes/extract-shared-script-toolkits/`。
- [ ] 1.4 审计 spec 是否覆盖通用内核/Docnav 策略分离、多包/多子仓库、typed config、发布准备、Docnav wrapper 行为保持和版本/changelog 控制。

## 2. 提取范围与基线

- [ ] 2.1 盘点 `scripts/tools/*`、`scripts/docnav-workspace/*`、`scripts/quality/*`、`scripts/release-package/*` 和 `scripts/tools/validators/*`，分类为 shared core、Docnav wrapper/config、暂不提取。
- [ ] 2.2 确定首批工具包或子仓库，优先选择边界清晰的基础 helper 和并行任务 runner。
- [ ] 2.3 记录暂不提取的能力及原因，尤其是仍绑定 Docnav validators、quality defaults、workspace profiles 或 release product config 的部分。
- [ ] 2.4 为每个首批工具包写明 public exports、runtime prerequisites、package manifest、verification scripts、version/changelog 策略和 Docnav integration path。
- [ ] 2.5 在迁移前记录 Docnav 侧基线命令、artifact paths、warning/status、report files、release manifest/checksum 和 exit behavior。

## 3. 工具包创建

- [ ] 3.1 创建基础脚本内核工具包，提取 process result/runner、Git helper、path/fs/json/args/type guard 等无 Docnav 产品语义的模块。
- [ ] 3.2 创建或迁移并行任务 runner 工具包，保留 task definition、dependency graph、concurrency、mutex/scheduler 和 completion hooks。
- [ ] 3.3 如首批包含 quality/verifier/release helper，只提取通用内核，并让 Docnav path、profile、validator、artifact 和 product metadata 留在 Docnav wrapper/config。
- [ ] 3.4 为每个工具包补充 README、exports、package scripts、typecheck/lint/test 配置、runtime prerequisites、package metadata 和 changelog 初始条目。

## 4. Docnav 集成层迁移

- [ ] 4.1 将 Docnav wrappers 改为调用共享工具包，同时保留 `scripts/quality/scan.ts`、`scripts/docnav-workspace/verify.ts`、`scripts/docnav-workspace/checks/*`、`scripts/release-package/*` 和 validators 的 Docnav owner 语义。
- [ ] 4.2 更新 pnpm workspace、package dependency、tsconfig、ESLint 或 Bun test 配置，使共享工具包和 Docnav wrappers 都被覆盖。
- [ ] 4.3 证明 `typecheck:scripts`、`lint:scripts`、`quality:check`、`verify:docnav-workspace`、release package scripts 和 validator scripts 的用途不变。
- [ ] 4.4 必要时更新 `docs/tooling.md`、`docs/testing.md` 或相关 owner 文档，说明共享工具包运行时前置条件和 Docnav wrapper 责任边界。

## 5. 子仓库与发布准备

- [ ] 5.1 为每个首批工具包记录 repository/package path、package name、exports、files/include policy、license/readme/changelog owner 和发布前检查入口。
- [ ] 5.2 为 Docnav 侧集成记录 workspace reference、git commit、submodule revision、package version 或等价 pin 机制。
- [ ] 5.3 验证 package manifest、exports、README、runtime prerequisites、typecheck/lint/test scripts、changelog 和发布说明完整。
- [ ] 5.4 记录回滚路径和首批未外移能力的后续演进任务。

## 6. 验证与交付审计

- [ ] 6.1 运行每个共享工具包的 typecheck、lint 和 focused tests，记录命令和结果。
- [ ] 6.2 运行 Docnav 相关脚本验证，至少覆盖 `bun run typecheck:scripts`、`bun run lint:scripts` 和受影响脚本 tests；触及 quality/verifier/release 时运行对应验证入口。
- [ ] 6.3 对比迁移前后的 Docnav command output、artifact paths、warning/status、report files、release manifest/checksum 和 exit behavior。
- [ ] 6.4 审计 Docnav CLI、adapter routing、protocol envelope、readable output、schemas 和 examples 没有因脚本提取发生可观察变化。
- [ ] 6.5 用局部 diff 审计实现只触及共享脚本工具提取相关 package/repo、Docnav wrappers、文档和验证材料。
