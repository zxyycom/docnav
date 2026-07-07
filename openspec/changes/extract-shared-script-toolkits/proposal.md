本 proposal 定义 `extract-shared-script-toolkits` change 的目标、范围和影响：把 Docnav 脚本工具中可复用的内核提取为一个或多个子仓库或可发布包，同时让 Docnav 专属配置、入口和验证规则继续留在 Docnav 集成层。

## Why

Docnav 当前的 TypeScript 脚本已经承担任务编排、进程执行、Git 辅助、质量扫描、workspace 验证和 release 包装等稳定职责。其中一部分并不绑定 Docnav 产品语义，具备被 Docnav 和其它项目复用的空间。

继续把通用内核和 Docnav 专属策略混放在 `scripts/` 下，会增加发布、版本演进和维护成本。本 change 只负责完成 Docnav 侧的提取边界、子仓库/包形态、发布准备和迁移验证；其它项目如何接入不属于本 change 的验收范围。

当前 change 只在 `openspec/changes/extract-shared-script-toolkits/` 下形成未审核临时文档，不影响既有文档或主规范。

## What Changes

- 新增 `shared-script-tooling` 能力，定义可提取的脚本工具内核、Docnav 专属留存边界和迁移门禁。
- 允许按能力拆成多个子仓库、包或 workspace package；不要求一个共享仓库覆盖所有脚本。
- 首批优先提取边界清晰的基础 helper、process/Git/path/fs/json/args 工具、并行任务 runner，以及已能分离 Docnav 策略的 quality/verifier/release helper 内核。
- 共享工具包通过 typed config、task definitions、tool adapters 或 explicit options 接收差异，不读取 Docnav 固定目录、artifact path、package name、OpenSpec layout 或 Cargo workspace shape。
- Docnav wrappers 继续拥有 `scripts/docnav-*`、quality default config、workspace check definitions、release product config、protocol/schema/examples validators、OpenSpec/docs validators 和 `package.json` scripts。
- 每个提取出的工具包必须具备发布准备：package manifest、public exports、README、runtime prerequisites、typecheck/lint/test scripts、changelog、版本或 pin 策略，以及 Docnav 侧回滚路径。

## Capabilities

### New Capabilities

- `shared-script-tooling`: 定义可复用 TypeScript 脚本工具包边界、配置注入要求、多包/多子仓库拆分策略、发布准备、Docnav 专属 owner 保留规则和迁移验证门禁。

### Modified Capabilities

- 无。现有 `repository-quality-observability` 继续拥有 Docnav 仓库质量观测语义，`release-artifacts` 继续拥有 Docnav release artifact 语义。

## Impact

- Affected script surface: `scripts/tools/process/*`、`scripts/tools/git.ts`、`scripts/tools/fs.ts`、`scripts/tools/path.ts`、`scripts/tools/args.ts`、`scripts/tools/json/*`、`scripts/tools/parallel-task-runner/*`、`scripts/tools/quality/*` 的通用内核、workspace verifier 的任务编排内核，以及部分 release helper。
- Affected Docnav-owned wrappers: `scripts/docnav-*`、`scripts/quality/scan.ts`、`scripts/docnav-workspace/checks/*`、`scripts/release-package/*`、`scripts/tools/validators/*` 和 `package.json` scripts。
- Affected validation: 共享工具包包级 typecheck/lint/tests、发布准备检查、Docnav 脚本迁移等价验证，以及迁移前后 command output、artifact path、warning/status、report 和 release metadata 对比。
