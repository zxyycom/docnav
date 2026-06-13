---
name: performance-optimization
description: "用于 performance optimization：当 measured budget、regression、profiling data 指向 CLI/API、parser/navigation、pagination、identifier lookup、bridge latency、output serialization、CPU 或 memory bottleneck 时使用。"
---

# Performance Optimization

## 使用边界

- 只在已有性能证据时使用：spec、OpenSpec、review 写出 budget，用户报告 regression，或 profiling/benchmark 指出慢路径。
- 默认面向本地工具和文档导航类 workload：CLI/API、parser/navigation、pagination、identifier lookup、output serialization、process/stdio overhead、CPU 或 memory 增长。
- 普通文档或小型代码改动不触发本 skill；只有已有 baseline、budget、regression report 或 profiling 证据时才进入性能流程。
- 不为直觉 micro-optimization 启动；没有 baseline 时，先建立 baseline。
- 不默认处理 Web performance。Core Web Vitals、Lighthouse、React、bundle、image optimization 只在用户明确要求 web UI 时参考。

## 先测量

每次改动前先记录可复现 workload：

- 命令/API：binary、subcommand、flags、path、output mode、page/limit、query、identifier/ref。
- Fixture：format、文件大小、heading 数量和深度、重复 heading、长 section、表格、代码块、frontmatter 等结构。
- 环境：debug/release、OS、存储位置、warm/cold cache、相关 env var。
- 结果：wall time、CPU/memory、stdout size、是否触发 pagination。

没有这些信息，不要声明 bottleneck 或收益。

## 工作流

1. Baseline：用当前 public surface 记录 before numbers。
2. Isolate：比较 direct implementation、CLI/API wrapper 和 bridge/tool path，把慢点归类为 parser/domain、routing、IO/process、output、identifier lookup、search、pagination、bridge 或 memory。
3. Fixture：用能代表问题的大文档、重复 heading、长 section、搜索和分页场景复现成本。
4. Benchmark：优先 release build；有 `hyperfine` 时用它，否则用 PowerShell `Measure-Command`；保持命令、fixture、output mode 和机器条件一致。
5. Fix：只改已证明的最小慢路径，保持 owner boundary、opaque identifier、schema、ordering、pagination 和 error behavior 稳定。
6. Remeasure：用同一 workload 比较 before/after；噪声较大时报告多次运行的 median 或保守范围。
7. Guard：加入覆盖优化路径的 unit test、smoke check、benchmark note 或 budget 文档。

## 参考加载

- 只有性能问题明确落在 Docnav 或类似文档导航、CLI/subprocess/bridge、large-document workload 或 workspace verification 时，读取 [performance-checklist.md](references/performance-checklist.md)。其中包含 scoped baseline 清单、fixture 形状、triage 表、命令模板、budget 模板、常见误判、red flags 和验证范围。
- `references/original-skill.md` 是迁移前来源记录；保持不变。只有在审计旧版通用 performance 内容时才读取。
- 不要继续追逐深层 reference；本 skill 的直接 reference 应足够完成项目专项性能工作。

## 验证

交付前确认：

- before/after measurements 使用同一 command、fixture、output mode、build profile 和环境假设。
- 已说明 bottleneck 分类，以及为什么当前改动命中该分类。
- User-visible behavior、opaque identifiers、pagination、schema、ordering 和 error mapping 没有被性能改动破坏。
- 已运行覆盖改动范围的最小 benchmark/test/smoke；跨语言/runtime、schema/examples、output layer、CLI/API、bridge 或 docs 边界时优先运行仓库约定的 workspace verification。
- 无法自动化 timing guard 时，已记录复现命令、fixture、budget 和原因。
