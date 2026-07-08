本 proposal 定义 `extract-shared-script-toolkits` change 的目标、范围和影响：把 Docnav 脚本工具中可复用的内核按能力拆成多个 Git 子仓库，同时让 Docnav 专属配置、入口和验证规则继续留在 Docnav 集成层。

## Why

Docnav 当前的 TypeScript 脚本已经承担任务编排、进程执行、Git 辅助、质量扫描、workspace 验证和 release 包装等稳定职责。其中一部分并不绑定 Docnav 产品语义，具备被 Docnav 和其它项目复用的空间。

继续把通用内核和 Docnav 专属策略混放在 `scripts/` 下，会增加版本演进、复用边界和维护成本。本 change 只负责完成 Docnav 侧的提取边界、多 Git 子仓库形态、交付准备和迁移验证；其它项目如何接入不属于本 change 的验收范围。

本 change 的 proposal、design、tasks 和 spec delta 由 `openspec/changes/extract-shared-script-toolkits/` 持有；归档前不修改主规范。

## What Changes

- 新增 `shared-script-tooling` 能力，定义可提取的脚本工具内核、Docnav 专属留存边界和迁移门禁。
- 首批物理成果固定为三个 Git 子仓库：`subrepos/script-foundation/`、`subrepos/script-parallel-task-runner/` 和 `subrepos/script-quality-core/`；这些子仓库不作为 npm package 发布，也不把 npm registry 作为集成路径。
- 首批共享边界固定为 foundation helper、parallel task runner 和 quality core；workspace verifier、release helper、validators、Docnav quality defaults 和 CLI wrappers 不进入首批共享默认值。
- 每个子仓库通过 public source entrypoint 暴露一个能力面；Docnav 通过子仓库 revision/pin 和源码 import 集成。
- 共享子仓库通过 typed config、task definitions、tool adapters 或 explicit options 接收差异，不读取 Docnav 固定目录、artifact path、OpenSpec layout 或 Cargo workspace shape。
- Docnav wrappers 继续拥有 `scripts/docnav-*`、quality default config、workspace check definitions、release product config、protocol/schema/examples validators、OpenSpec/docs validators 和 `package.json` scripts。
- 每个子仓库必须具备交付准备：private tooling manifest、public source entrypoint、README、runtime prerequisites、typecheck/lint/test scripts、changelog、Git revision/pin 策略，以及 Docnav 侧回滚路径。

## Capabilities

### New Capabilities

- `shared-script-tooling`: 定义可复用 TypeScript 脚本工具的多 Git 子仓库边界、配置注入要求、public source entrypoint 策略、交付准备、Docnav 专属 owner 保留规则和迁移验证门禁。

### Modified Capabilities

- 无。现有 `repository-quality-observability` 继续拥有 Docnav 仓库质量观测语义，`release-artifacts` 继续拥有 Docnav release artifact 语义。

## Impact

- Affected script surface: 新增 `subrepos/script-foundation/`、`subrepos/script-parallel-task-runner/` 和 `subrepos/script-quality-core/` Git 子仓库；迁移 `scripts/tools/process/*`、`scripts/tools/git.ts`、`scripts/tools/fs.ts`、`scripts/tools/path.ts`、`scripts/tools/args.ts`、`scripts/tools/json/*`、`scripts/tools/type-guards.ts`、`scripts/tools/parallel-task-runner/*` 和 `scripts/tools/quality/*` 的通用内核。
- Affected Docnav-owned wrappers: `scripts/docnav-*`、`scripts/quality/scan.ts`、`scripts/docnav-workspace/checks/*`、`scripts/release-package/*`、`scripts/tools/validators/*` 和 `package.json` scripts。
- Affected validation: 每个子仓库的 typecheck/lint/tests、交付准备检查、Docnav 脚本迁移等价验证，以及迁移前后 command output、artifact path、warning/status、report 和 quality artifacts 对比。
