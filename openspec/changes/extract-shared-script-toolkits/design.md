本 design 定义 `extract-shared-script-toolkits` 的实现前决策：按能力边界提取 Docnav 脚本工具内核，并把 Docnav 专属策略保留在 Docnav 集成层。

## Context

Docnav 脚本体系已经承担稳定工程职责。`docs/tooling.md` 规定手写 TypeScript 脚本以 `.ts` 源码为目标形态，通过 Bun 运行、pnpm 管理依赖、`tsgo` 做类型检查、ESLint 做静态质量检查；`docs/testing.md` 规定 `typecheck:scripts` 和 `lint:scripts` 证明脚本模块 contract 与静态质量边界。

当前脚本里已经存在较清晰的可复用内核：`scripts/tools/parallel-task-runner` 提供 task definition、依赖图、concurrency 和 scheduler；`scripts/tools/process` 封装 process result；`scripts/tools/git.ts`、`args.ts`、`path.ts`、`fs.ts` 等基础 helper 没有强 Docnav 产品语义。

也存在必须留在 Docnav 的策略：`scripts/tools/quality/model/config.ts` 绑定 Docnav code areas、warning policy 和 artifact directory；`scripts/docnav-workspace/checks/definitions.ts` 绑定 Docnav required/full profiles、Cargo/OpenSpec/docs validators 和 `.cache/docnav`/`artifacts/docnav-quality`；`scripts/release-package/config.ts` 绑定 `product: "docnav"`、`binName: "docnav"` 和 Docnav artifact layout；`scripts/tools/validators/*` 多数绑定 Docnav protocol/schema/examples/docs。

## Goals / Non-Goals

**Goals:**

- 提取 Docnav 脚本中可复用的工具内核，使其具备被 Docnav 和其它项目复用的包边界。
- 按能力、成熟度和发布节奏拆成一个或多个子仓库、包或 workspace package。
- 用 typed config、task definitions、tool adapters 和 explicit options 表达可变策略。
- 保持 Docnav wrappers、package scripts、artifact paths、warning policy、verification profiles 和 release package 行为等价。
- 为每个首批工具包补齐发布准备和 Docnav 侧 pin/rollback 路径。

**Non-Goals:**

- 不改变 Docnav core CLI、adapter、protocol、schema、examples 或 document output contract。
- 不把 Docnav validators、quality defaults、workspace profiles 或 release product config 做成共享默认值。
- 不要求一次性抽出整个 `scripts/` 目录，也不要求首批完成所有候选工具包。
- 不在本 change 中实现其它项目接入；复用能力只作为提取动机和包边界要求。

## Decisions

### Decision 1: 按能力边界拆分工具包

首批候选边界是基础脚本内核和并行任务 runner。Quality、workspace verifier 和 release helper 只有在能清楚分离 Docnav 策略时才进入首批；否则保留在 Docnav 脚本中并记录原因。

### Decision 2: 共享包只拥有通用内核

共享包不得隐式读取 Docnav root、`artifacts/docnav-*`、`.cache/docnav/*`、`docs/schemas`、`docs/examples`、OpenSpec layout、`crates/**/*.rs` 或 `package.json` scripts。Docnav 专属规则由 Docnav wrapper 或配置传入。

### Decision 3: 发布边界和代码边界一起交付

每个提取出的工具包必须有 package manifest、public exports、README、runtime prerequisites、typecheck/lint/test scripts、changelog、版本或 pin 策略。物理形态可以是 workspace package、git dependency、submodule/subtree、独立子仓库或 registry package，但必须记录选择理由和回滚路径。

### Decision 4: Docnav 集成层保持可观察行为

Docnav wrappers 继续拥有 command name、profile composition、artifact path、schema/examples validation 和 release package contract。迁移必须先记录基线，再对比迁移后的 command output、artifact path、warning/status、report 和 release metadata。

## Risks / Trade-offs

- [Risk] API 过早扩大。Mitigation: 首批只暴露实现迁移需要的最小 exports，后续扩展走 changelog 和版本更新。
- [Risk] Docnav 策略进入共享默认值。Mitigation: 所有 Docnav path、profile、validator、artifact 和 product metadata 只允许出现在 Docnav wrapper/config。
- [Risk] 子仓库过多增加维护成本。Mitigation: 只有生命周期、依赖或发布节奏明显不同的能力才拆独立边界。
- [Risk] 行为等价难以判断。Mitigation: 迁移前记录基线，交付时对比输出、artifact、warning/status 和 exit behavior。

## Migration Plan

1. 完成阻塞级审计，确认 proposal、design、spec 和 tasks 只围绕脚本工具提取。
2. 盘点脚本文件，分类为 shared core、Docnav wrapper/config、暂不提取。
3. 选择首批工具包边界，并记录物理分发方式、发布准备和 Docnav 集成方式。
4. 记录迁移前 Docnav 脚本基线。
5. 创建共享工具包并迁移 Docnav wrappers。
6. 运行包级验证、发布准备检查和 Docnav 等价验证。

## Open Questions

无未回答开放问题，可以进入实现前审计。实际实现时仍需在审计任务中选择首批工具包和物理仓库形态。
