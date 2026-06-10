---
name: source-driven-development
description: 以 official docs 和 source citation 驱动实现决策，避免过时模式；适用于需要 authoritative、verified、documented、source-cited 的 framework、library、API 或 correctness-sensitive implementation。
---

# 源驱动开发 (Source-Driven Development)

## 目标

所有依赖 framework、library、runtime、standard 或 public API 的关键实现决策，都要先验证当前版本的 official source，再实现并给出可检查 citation。不要用记忆、流行 blog、Stack Overflow 或 AI summary 代替来源。

## 最小流程

1. Detect stack：
   - 读取 dependency file 或 lockfile，识别相关 framework、library、runtime 和版本。
   - 版本缺失且影响 pattern 正确性时，问用户或标记假设。
   - 只为当前任务识别相关技术栈，不做全仓库技术盘点。

2. Select sources：
   - 优先级：official documentation -> official blog/changelog/migration guide -> web standards -> browser/runtime compatibility data。
   - 只获取与当前 pattern 直接相关的页面，优先 deep link。
   - 不把 blog、tutorial、Q&A、AI summary 或训练记忆作为 primary source。

3. Read and extract：
   - 提取 API signature、版本条件、deprecation、migration guidance、示例 pattern 和兼容性要求。
   - official sources 冲突时，按检测到的版本判断；无法判断就向用户展示冲突。

4. Implement：
   - 按 source 展示的当前 pattern 写代码。
   - 与现有代码风格冲突时，明确说明权衡；需要产品或维护策略选择时问用户。
   - source 未覆盖的部分标记为 unverified，不伪装成已验证事实。

5. Cite：
   - 在最终回复列出影响关键决策的 URLs。
   - 非显而易见的代码内选择可以加短注释和 source URL。
   - citation 要能让用户复查，避免只给 homepage。

## Reference 读取策略

- source hierarchy、citation format、unverified 标记和冲突处理：读 [source-rules.md](references/source-rules.md)。
- 版本探测、实现检查、red flags 和验收清单：读 [workflow-checks.md](references/workflow-checks.md)。

只读取当前任务需要的 reference。不要为了一个局部 API 用法抓取整站文档。

## 项目契约边界

- official source hierarchy 适用于外部技术决策；项目自身 contract 以本仓库的主规范和 owner 文档为准。
- Schema、examples、fixtures 和 tests 是验证材料或证据，不应被当作新的规则来源。
- 当外部 docs 与项目已有约定冲突时，先保护本仓库 contract，再说明冲突和可选迁移路径。

## 验收

完成 source-driven work 后确认：

1. 已识别相关技术栈和版本，或明确说明版本假设。
2. 关键 framework/library/API pattern 有 official source citation。
3. 未使用已知 deprecated API，或已说明兼容性原因。
4. docs 与现有代码冲突时已显式处理。
5. 无法验证的内容已标记为 unverified。
6. 最终回复包含用户可打开的 source URLs。
