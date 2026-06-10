本 design 仅为 `shorten-markdown-refs` change 的未审核临时文档，核心目标是定义 Markdown adapter 短 ref 的 breaking migration 方案。

本 change 只在 `openspec/changes/shorten-markdown-refs/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## Context

Docnav 的共享协议、`docnav` core 和 MCP 都把 ref 作为 opaque string 原样传递；Markdown ref 的生成、解析和兼容性由 `docnav-markdown` adapter 拥有。
当前 Markdown heading ref 使用 `L{line}:{path}` / `L{line}#{ordinal}:{path}`，把完整 heading breadcrumb 放入命令行，导致深层文档或长标题场景中的 `--ref` 过长。

本 change 是 breaking migration：实现后调用方必须重新执行 `outline` 或 `find` 获取新 ref，旧长 ref 不再被 `read` 接受。

## Goals / Non-Goals

**Goals:**

- 将 Markdown heading canonical ref 改为短、ASCII、命令行友好的 adapter 自有格式。
- 保持同一文档内 heading ref 非空、唯一，并可被 `read` 原样消费。
- 让 `outline`、`find` 和 `read` 使用同一套短 ref 生成与解析规则。
- 更新相关文档、schema 示例、fixture、golden output 和测试，明确这是 breaking migration。

**Non-Goals:**

- 不为旧 `L{line}:{path}` / `L{line}#{ordinal}:{path}` heading ref 提供读取兼容。
- 不保留旧 `doc:full` 全文 fallback ref。
- 不新增 `--like`、fuzzy resolve、自动 fallback 或 core/MCP 侧 ref 解析。
- 不改变 protocol result 字段 shape、错误 code、分页模型或 adapter routing。

## Decisions

1. Markdown heading ref 使用 `H{line}:{token}`。

   `H` 表示 Markdown heading ref，`line` 是 1-based heading 起始行号，`token` 是由 canonical heading breadcrumb 和 occurrence ordinal 派生的短 token。该格式比旧格式短，并且仍保留行号作为人类可读定位提示和解析校验输入。

   Alternatives considered:
   - 纯 ordinal，例如 `H3`：最短，但文档插入 heading 后更容易误读到错误区域。
   - 纯 hash，例如 `H:9q4f2k`：短，但失去行号提示，不利于文本输出审计。
   - 保留旧格式并新增 alias：非 breaking，但不能解决旧长 ref 继续传播的问题。

2. `token` 由 adapter 在当前文档内生成并唯一化。

   初始 token 使用稳定摘要的短前缀，摘要输入包含 canonical heading breadcrumb 和 occurrence ordinal；如果当前文档内发生 token 冲突，adapter 必须扩展 token 长度直到唯一。实现不得使用时间戳、随机数、文件系统绝对路径或其它不稳定输入。

   Alternatives considered:
   - 固定长度 hash：实现简单，但理论碰撞时无法满足唯一性契约。
   - 暴露完整 digest：唯一性强，但会重新拉长 ref。

3. `read` 只接受新短格式。

   Markdown `read` 必须解析 `H{line}:{token}` 并匹配唯一 heading；无匹配返回 `REF_NOT_FOUND`，多匹配返回 `REF_AMBIGUOUS`。旧 `L...` heading ref 和旧全文 `doc:full` 必须返回稳定 ref 错误，不做兼容解析。

   Alternatives considered:
   - 接受旧格式作为 legacy alias：迁移平滑，但与本 change 的完全迁移目标冲突。
   - 在 core 中转换旧 ref：违反 ref opaque 边界。

4. 全文 fallback ref 改为 `D`。

   当当前 outline 参数下没有 heading entry 时，Markdown adapter 生成单条全文 entry，ref 为 `D`。`D` 由 Markdown adapter 拥有和解析，表示读取整篇 Markdown 文档。

   Alternatives considered:
   - 保留 `doc:full`：兼容但不是完全迁移。
   - 使用 `H0:...`：会混淆 heading ref 和全文 ref。

## Risks / Trade-offs

- [Risk] 已复制的旧 ref 会失效。→ Mitigation：proposal、spec、tasks 和 release note 明确 breaking migration；测试断言旧 ref 返回稳定 ref 错误。
- [Risk] token 过短时存在碰撞。→ Mitigation：生成阶段检查当前文档内唯一性，冲突时扩展 token 长度。
- [Risk] 文档变更后短 ref 仍可能失效。→ Mitigation：保持既有规则，文档变化后调用方重新执行 `outline` 或 `find`。
- [Risk] 示例和 golden output 分散，容易遗漏旧 ref。→ Mitigation：tasks 增加阻塞级审计和全仓旧 ref 搜索更新任务。

## Migration Plan

1. 先完成阻塞级审计，确认 proposal、design、tasks 和 delta spec 都围绕短 ref breaking migration，且 change 目录外没有被本提案阶段修改。
2. 更新 Markdown ref 生成和解析实现，使 `outline`、`find`、`read` 统一使用 `H{line}:{token}` 和 `D`。
3. 更新 Markdown adapter 单元测试、CLI smoke、负向矩阵、fixture/golden output、文档和示例。
4. 运行局部验证，再按范围运行 `pnpm run verify:docnav-workspace`。

Rollback strategy：该 change 是 breaking migration；如果实现后需要回退，应整体回退本 change 的实现和验证材料，而不是在同一实现中恢复 legacy alias。
