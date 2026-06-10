# 性能清单（Performance Checklist）

这是 code review 时使用的 Docnav performance quick reference。只有当改动触及 hot path、large document behavior、pagination、adapter invocation、serialization、MCP bridge 或可能引入 regression 时才加载。

性能判断必须以 measurement 或明确复杂度变化为依据。不要把主观“看起来更快/更慢”当作 finding。

## 审查入口（Review Entry Points）

- [ ] 改动是否影响 `outline -> ref -> read` 的常用路径。
- [ ] 是否改变 Markdown parsing、heading scan、frontmatter handling、ref lookup、pagination 或 continuation。
- [ ] 是否引入 full-document load、重复 parse、N+1 traversal、unbounded recursion 或大对象 clone。
- [ ] 是否改变 adapter process startup、stdio JSON envelope、output serialization 或 MCP bridge spawning。
- [ ] 是否影响 Windows path handling、large file streaming、stdout/stderr capture 或 timeout behavior。

## 先度量（Measurement First）

- [ ] 有 baseline：现有 test、fixture、smoke command、benchmark、profiling data 或复杂度说明。
- [ ] 有对比：改动前后使用相同文档、相同 command、相同 output mode 与相同 limits。
- [ ] 结果报告 p50/p95 或至少多次运行的稳定趋势；单次 wall-clock 只作为弱证据。
- [ ] 如果没有可运行 measurement，review 说明 residual risk，并要求聚焦 fixture 或 follow-up。

## 大文档与解析（Large Documents / Parsing）

- [ ] Parsing 不会在每个 page、match 或 ref read 中重复做完整扫描，除非有明确缓存或成本证明。
- [ ] Heading/frontmatter/code block scan 是线性或有界行为，避免嵌套循环随 heading count/document size 爆炸。
- [ ] Large code blocks、long lines、invalid Unicode 与 deeply nested headings 不会导致 pathological memory growth。
- [ ] 读取摘要、outline 或 find 时，不会为了 readable output 构造不需要的大段正文。
- [ ] 需要保留 source slice 时，避免无意义复制；但不要为微优化牺牲 ref/protocol correctness。

## Refs、Pagination 与 Continuation（继续读取）

- [ ] `read` 按 ref 定位区域时不做不必要的全量 materialization。
- [ ] `outline`、`find` 与 `read` 都强制 `limit_chars`、page size 或 output caps。
- [ ] Pagination metadata 稳定，continuation 不会跳项、重复项或需要重新加载无限结果集。
- [ ] Adapter-owned ref lookup 不引入跨文档全局搜索或不受控缓存。
- [ ] Error path 与 empty result path 同样有界，不能因为没有命中而扫描额外无关资源。

## CLI、Adapter 与 MCP Bridge（桥接）

- [ ] Core `docnav` 没有复制 adapter parsing/routing 逻辑；重复实现通常也会重复成本。
- [ ] Adapter process invocation 有 timeout 与 output-size caps；失败路径不会等待额外 stderr/stdout。
- [ ] JSON serialization/deserialization 不重复转换 raw/readable/protocol wrappers。
- [ ] MCP bridge 保持 thin，不在 Node 层重新 parse 文档或重建 pagination。
- [ ] stdout/stderr logging 不输出大块 document body，也不在 hot path 做昂贵 formatting。

## 依赖与构建成本（Dependency / Build Cost）

- [ ] 新 dependency 对 runtime cost、binary size、install/build time 与 platform support 的影响被说明。
- [ ] Node dependency 改动使用 `pnpm` workflow；Rust dependency 改动使用 repo-approved Cargo checks。
- [ ] Generated artifacts 不显著放大 repo size、test fixture size 或 CI runtime。

## 验证命令（Verification Commands）

根据改动范围选择最小能证明风险的命令：

```text
Markdown skill/reference edits:
  D:\project\ai\docnav\target\debug\docnav-markdown.exe info <changed-md>
  D:\project\ai\docnav\target\debug\docnav-markdown.exe outline <changed-md>

Markdown adapter behavior:
  pnpm run smoke:docnav-markdown

Core CLI / adapter routing:
  pnpm run smoke:docnav-core

Cross-boundary protocol/schema/MCP behavior:
  pnpm run verify:docnav-workspace
```

## 常见反模式（Anti-Patterns）

| Anti-pattern | Impact | Review response |
| --- | --- | --- |
| Full-document load for every page | Large docs become slow and memory-heavy | Require bounded pagination or cached parse |
| Parsing refs outside adapter | Duplicates logic and cost, breaks ownership | Keep ref opaque outside owning adapter |
| Rebuilding raw/readable wrappers repeatedly | Serialization overhead and contract risk | Share business data, not transport wrappers |
| Unbounded `find` or recursive traversal | Timeouts and high memory usage | Require limits and negative fixtures |
| Logging raw document bodies | Slow output and possible secret leakage | Log concise structured context |
| Adding dependency for tiny helper | Install/build/runtime cost | Prefer local simple code when clear |
| Re-running broad verification unchanged | Wastes review time | Run again only after relevant edits |
