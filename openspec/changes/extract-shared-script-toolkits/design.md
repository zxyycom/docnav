本 design 定义 `extract-shared-script-toolkits` 的实现前决策：按能力边界把 Docnav 脚本工具内核提取到多个 Git 子仓库，并把 Docnav 专属策略保留在 Docnav 集成层。

## Context

Docnav 脚本体系已经承担稳定工程职责。`docs/tooling.md` 规定手写 TypeScript 脚本以 `.ts` 源码为目标形态，通过 Bun 运行、pnpm 管理依赖、`tsgo` 做类型检查、ESLint 做静态质量检查；`docs/testing.md` 规定 `typecheck:scripts` 和 `lint:scripts` 证明脚本模块 contract 与静态质量边界。

当前脚本里已经存在较清晰的可复用内核：`scripts/tools/parallel-task-runner` 提供 task definition、依赖图、concurrency 和 scheduler；`scripts/tools/process` 封装 process result；`scripts/tools/git.ts`、`args.ts`、`path.ts`、`fs.ts` 等基础 helper 没有强 Docnav 产品语义。

也存在必须留在 Docnav 的策略：`scripts/tools/quality/model/config.ts` 绑定 Docnav code areas、warning policy 和 artifact directory；`scripts/docnav-workspace/checks/definitions.ts` 绑定 Docnav required/full profiles、Cargo/OpenSpec/docs validators 和 `.cache/docnav`/`artifacts/docnav-quality`；`scripts/release-package/config.ts` 绑定 `product: "docnav"`、`binName: "docnav"` 和 Docnav artifact layout；`scripts/tools/validators/*` 多数绑定 Docnav protocol/schema/examples/docs。

## Goals / Non-Goals

**Goals:**

- 提取 Docnav 脚本中可复用的工具内核，使其具备被 Docnav 和其它项目复用的多 Git 子仓库边界。
- 首批固定为三个 Git 子仓库：`subrepos/script-foundation/`、`subrepos/script-parallel-task-runner/` 和 `subrepos/script-quality-core/`，分别拥有 foundation helper、parallel task runner 和 quality core。
- 用 typed config、task definitions、tool adapters 和 explicit options 表达可变策略。
- 保持 Docnav wrappers、package scripts、artifact paths、warning policy、verification profiles 和 release package 行为等价。
- 为每个子仓库补齐交付准备和 Docnav 侧 submodule revision/pin/rollback 路径。

**Non-Goals:**

- 不改变 Docnav core CLI、adapter、protocol、schema、examples 或 document output contract。
- 不把 Docnav validators、quality defaults、workspace profiles 或 release product config 做成共享默认值。
- 不要求一次性抽出整个 `scripts/` 目录，也不要求首批完成所有候选能力。
- 不发布 npm package，不把 npm registry 或 package exports map 作为本 change 的集成路径。
- 不在本 change 中实现其它项目接入；复用能力只作为提取动机和包边界要求。

## Decisions

### Decision 1: 首批边界固定为 helper、runner 和 quality core

首批共享边界固定为 foundation helper、parallel task runner 和 quality core。Workspace verifier、release helper、validators、Docnav quality defaults 和 CLI wrappers 不进入首批共享默认值；它们只作为 Docnav 集成层调用共享能力。

### Decision 2: 共享子仓库只拥有通用内核

共享子仓库不得隐式读取 Docnav root、`artifacts/docnav-*`、`.cache/docnav/*`、`docs/schemas`、`docs/examples`、OpenSpec layout、`crates/**/*.rs` 或 `package.json` scripts。Docnav 专属规则由 Docnav wrapper 或配置传入。

### Decision 3: 物理成果是多个 Git 子仓库，不是 npm package

物理成果固定为三个 Git 子仓库：`subrepos/script-foundation/`、`subrepos/script-parallel-task-runner/` 和 `subrepos/script-quality-core/`。每个子仓库可以有 private `package.json` 作为 Bun/TypeScript/ESLint/test 的脚本 manifest，但不得把 npm package 发布、registry package version 或 package exports map 作为 Docnav 集成契约。Docnav 通过每个子仓库的 submodule revision 或等价 Git pin 记录集成状态和回滚路径。

### Decision 4: Docnav 集成层保持可观察行为

Docnav wrappers 继续拥有 command name、profile composition、artifact path、schema/examples validation 和 release package contract。迁移必须先记录基线，再对比迁移后的 command output、artifact path、warning/status、report 和 quality artifacts。

### Decision 5: 每个子仓库拥有一个 public source entrypoint

每个子仓库只暴露自己的稳定 public source entrypoint。`script-foundation` 拥有 process result/runner、generic Git client、path/fs/json/args/type guard 等无 Docnav 产品语义的 helper；`script-parallel-task-runner` 拥有 task definition、normalization、dependency graph、concurrency、mutex scheduler 和 lifecycle hooks；`script-quality-core` 拥有 quality schema/types、code-area classification mechanism、scanner adapters/parsers、metrics aggregation、warning/report generation、baseline/cache primitives 和 `runQualityScan` engine API。`script-parallel-task-runner` 和 `script-quality-core` 只能通过 Git pin 依赖 `script-foundation`，不得反向依赖或形成循环依赖。

### Decision 6: Quality 配置由 Docnav wrapper 注入

共享 quality core 不导出 Docnav `DEFAULT_CONFIG`，也不拥有 `DOCNAV_*` 环境变量、`artifacts/docnav-quality`、`.cache/docnav/quality`、`crates/**/*.rs`、`docs/examples/**` 或 `docs/schemas/**` 默认值。Docnav 保留 `scripts/tools/quality/model/config.ts` 和 `scripts/quality/scan.ts`，并通过 typed config/options 向 `runQualityScan` 传入 root、artifact dir、cache dir、code areas、include/exclude globs、thresholds、accepted warnings、tool commands、profile、changed files、baseline policy、report options 和 output adapter。

## Risks / Trade-offs

- [Risk] API 过早扩大。Mitigation: 首批只暴露实现迁移需要的最小 exports，后续扩展走 changelog 和版本更新。
- [Risk] Docnav 策略进入共享默认值。Mitigation: 所有 Docnav path、profile、validator、artifact 和 product metadata 只允许出现在 Docnav wrapper/config。
- [Risk] 多子仓库增加 pin 和验证成本。Mitigation: 首批只拆 foundation、parallel task runner 和 quality core 三个边界；跨子仓库依赖只允许从 runner/quality 指向 foundation，并记录每个 revision/pin、验证入口和回滚路径。
- [Risk] 行为等价难以判断。Mitigation: 迁移前记录基线，交付时对比输出、artifact、warning/status 和 exit behavior。

## Migration Plan

1. 完成阻塞级审计，确认 proposal、design、spec 和 tasks 只围绕脚本工具提取。
2. 盘点脚本文件，分类为 shared core、Docnav wrapper/config、暂不提取。
3. 创建 `subrepos/script-foundation/`、`subrepos/script-parallel-task-runner/` 和 `subrepos/script-quality-core/` Git 子仓库边界，并记录 public source entrypoint、交付准备和 Docnav 集成方式。
4. 记录迁移前 Docnav 脚本基线。
5. 创建共享子仓库并迁移 Docnav wrappers。
6. 运行每个子仓库的验证、交付准备检查和 Docnav 等价验证。

## Open Questions

无未回答开放问题。首批共享边界和物理形态已固定为 `subrepos/script-foundation/`、`subrepos/script-parallel-task-runner/` 和 `subrepos/script-quality-core/` 三个 Git 子仓库。
