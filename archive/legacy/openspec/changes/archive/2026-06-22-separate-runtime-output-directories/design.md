## Context

当前 `.log/` 同时包含 smoke 审计日志、workspace verifier 完整输出、quality duplicate-code cache、smoke 临时目录、verifier dev binary env file，以及若干人工指定的 quality scan 报告目录。`.cache/` 已存在并承载 TypeScript incremental build info，但 Docnav 自有缓存没有使用它；`artifacts/docnav-quality/` 已是 quality quick check 的默认报告目录。

本 change 只影响仓库验证和开发工具链的本地运行期输出位置，不改变 `docnav` CLI、Markdown adapter 业务行为、protocol JSON、readable output、schema 或 MCP bridge 映射。

## Goals / Non-Goals

**Goals:**

- 让 `.log/` 只表达可审计文本日志，路径按 owner 分组。
- 新增 `.tmp/` 并把一次性 scratch workspace 从 `.log/` 拆出。
- 把可复用缓存和运行中间状态放入 `.cache/docnav/`。
- 保持 quality scan 报告归属 `artifacts/docnav-quality/`。
- 更新 docs、OpenSpec delta、测试期望和 ignore/exclude 配置。

**Non-Goals:**

- 不迁移已有本地 `.log/` 历史内容。
- 不提供旧路径兼容读取。
- 不改变发布制品布局、CLI 参数、adapter protocol 或 MCP 行为。

## Decisions

1. 使用 `.tmp/` 承载 scratch workspace。
   - 理由：`.cache` 表达可复用、命中可加速的数据；smoke 临时 project/workspace 是单次运行副产物，删除后不应影响性能语义。
   - 备选：把所有可删除数据都放入 `.cache/docnav/`。该方案目录更少，但会混淆缓存和临时工作区的维护策略。

2. 使用 owner-first 目录分组。
   - 理由：`.log/smoke/markdown/` 和 `.log/smoke/core/` 把 smoke 归属表达为目录层级，避免同一系列输出在顶层用前缀平铺。
   - 备选：保留原目录名并只移动 cache/tmp。该方案改动小，但不能解决 `.log` 顶层混乱。

3. 当前脚本只读写新 runtime layout。
   - 理由：这些目录均被 `.gitignore` 忽略且只保存本地诊断、缓存和临时产物。cache miss 只会重新计算；旧 logs/reports 是历史诊断材料，不应成为当前脚本输入。
   - 备选：做一个过渡期双写或 fallback。该方案会增加测试矩阵和长期维护成本，但收益很低。

4. 将 verifier dev binary env file 移入 `.cache/docnav/verify/`。
   - 理由：该 JSON 是同一次 workspace verification 的进程间中间状态，不是可审计日志。
   - 备选：放入 `.tmp/`。该文件在同一 verifier 执行内由后续任务读取，语义接近运行状态；`.cache/docnav/verify/` 让它和 verifier owner 绑定更清晰。

## Risks / Trade-offs

- 测试输出过滤仍匹配旧路径，导致 verifier 将正常输出视为噪声或失败。→ 更新 `smokeSuccessOutput` 期望和 verifier tests，并运行 workspace verifier script tests。
- quality cache 首次使用新路径时发生 cache miss。→ 接受一次性重新计算；cache payload 已定义 miss 不影响 correctness。
- 新 `.tmp/` 被扫描或 lint 纳入。→ 更新 `.gitignore`、ESLint ignores 和 quality exclude dirs。
- 文档和 OpenSpec 主规范路径不一致。→ 同步 `docs/testing.md`、`openspec/specs/*` delta，并运行 OpenSpec validation。

## Migration Plan

1. 更新路径常量和默认输出路径。
2. 更新测试和输出过滤期望。
3. 更新 docs/OpenSpec 文本与 ignore/exclude 配置。
4. 运行 targeted tests、OpenSpec validation 和必要的 workspace verification。
5. 本地历史 `.log/` 内容不迁移；维护者可按需删除旧目录。

## Open Questions

- 无。
