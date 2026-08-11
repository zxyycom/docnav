**一句话核心：Markdown adapter 只负责 Markdown 格式语义，并通过已实现的协议/SDK 输出稳定 invoke 结果。**

## Context

Markdown 是 v0 首期唯一实现格式。主规范要求 Markdown adapter 独立于 `docnav` 核心 CLI，拥有 parser、导航策略、ref 生成解析、分页和 adapter 直接 CLI 输出。

## Goals / Non-Goals

**Goals:**

- 实现 `docnav-markdown` 的 `manifest`、`probe`、`invoke` 和直接 CLI 文档操作。
- 实现 `outline`、`read`、`find`、`info` 全部能力。
- 保证 outline 扁平、ref 可读且唯一、read 能唯一定位。
- 实现 Markdown 默认值：outline/read/find 每页 6000 字符，outline 默认 H1-H3，find 使用同一 heading 可见性选择最近 outline ref。
- 覆盖 MarkdownNavigator 参考基线中的边界案例。

**Non-Goals:**

- 不实现 `docnav` 核心 CLI 的跨 adapter 选择。
- 不实现 adapter 安装、更新、移除和列表管理。
- 不实现其它格式 adapter。

## Decisions

1. 使用成熟 Markdown parser 并保留源码位置。
   - 理由：frontmatter、代码围栏伪 heading、嵌套章节和重复 heading 容易被字符串扫描误判。
   - 替代方案：手写行扫描；拒绝，因为边界案例风险高。

2. ref 格式完全由 Markdown adapter 拥有。
   - ref 可包含行号、heading path 和重复项序号等定位证据，但 `docnav` 和 MCP 只原样传递。
   - ref 格式必须在 adapter 内部有解析和错误测试。

3. read 以 heading 节点范围为基本区域。
   - 章节从目标 heading 开始，到下一个同级或更高级 heading 前结束。
   - 若当前 outline 参数无法产生任何 entry，adapter 生成全文 ref；read 读取该 ref 时返回整篇 Markdown 文档。

4. 分页使用共享字符预算工具，但预算对象由 operation 决定。
   - outline/find 按 `ref + display` 计入预算。
   - read 按 `content` 计入预算。
   - page 非 null 时固定为请求 page 加 1。

5. find 的 match ref 指向最近 outline entry。
   - find 搜索全文，但每个 match 的 ref 使用同一 heading 可见性下离命中位置最近的 outline entry。
   - 若当前 outline 参数没有任何 entry，find 使用同一个全文 ref。
   - find 不为每个命中生成临时片段 ref，也不把所有 match 默认归到全文 ref。

6. 直接 CLI 和 invoke 复用业务逻辑，不复用输出包装。
   - `invoke` 输出完整 protocol envelope。
   - adapter 直接 CLI 的 text/readable-json/protocol-json 按 adapter 契约输出。

## Risks / Trade-offs

- [ref 可读性与稳定定位冲突] → ref 同时保留可读 heading path 和足够消歧信息，测试重复完整路径。
- [字符预算切分页破坏 Unicode] → 分页按 Unicode 字符计数，禁止切断字符。
- [parser 行为与参考基线不一致] → 使用 fixture 固化保留、调整、推迟和移除的行为。

## Migration Plan

1. 在协议/SDK change 完成后实现 adapter。
2. 先完成 manifest/probe，再完成 outline/read，最后补 find/info 和直接 CLI 输出。
3. 与核心 CLI change 联调 `docnav outline -> ref -> read`。

## Open Questions

- 具体 Markdown parser crate 或库在实现时根据仓库语言栈和可维护性选择，但必须满足源码位置与 CommonMark 边界需求。
