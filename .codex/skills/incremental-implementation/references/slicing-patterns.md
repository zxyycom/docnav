# Slicing Patterns

本引用用于需要具体 slicing examples、feature flag、rollback 或 Docnav vertical slice 设计时。

## Vertical Slice

优先选择贯穿真实路径的最小可用功能，而不是按层批量实现。

通用例子：

```text
Slice 1: Create task, including storage + API + minimal UI
Slice 2: List tasks, including query + API + UI state
Slice 3: Edit task, including update path + validation + UI action
Slice 4: Delete task, including confirmation + removal path
```

Docnav Markdown 例子：

```text
Slice 1: Parser 识别一种 heading edge case，并用 adapter unit test 证明。
Slice 2: `outline` 输出包含该 heading，并检查 `protocol-json`。
Slice 3: `read` 可使用生成 ref 读取该 region。
Slice 4: Core CLI 和 MCP smoke 覆盖同一路径。
```

每片都应该能用一个命令或一个 replay path 证明。

## Contract-First Slice

当多个 worker 要并行时，先冻结 shared contract：

```text
Slice 0: 更新 protocol/schema/types 和最小 example。
Slice 1a: Rust adapter/core 按 contract 实现。
Slice 1b: Node MCP bridge 按 contract 映射 tool args/result。
Slice 2: 运行 schema、smoke 和 workspace verification 做集成。
```

Contract slice 的完成条件是字段名、error shape、pagination/continuation 语义和 examples 可验证。

## Risk-First Slice

当风险集中在未知边界时，先证明风险最高的假设：

- Windows path quoting 是否能穿过 CLI -> adapter process。
- `invoke` stdin JSON envelope 是否能被 adapter protocol dispatch。
- `limit-chars` 与 multibyte text 是否会破坏 pagination。
- generated fixture 是否来自 generator、source document 还是 implementation drift。

Risk slice 失败时，把发现写进下一片，不扩大实现范围。

## Feature Flag And Safe Defaults

未完成但需要合并的能力采用 opt-in：

```typescript
const ENABLE_MARKDOWN_V2_REFS = process.env.DOCNAV_MARKDOWN_V2_REFS === "true";

if (ENABLE_MARKDOWN_V2_REFS) {
  // New ref behavior
}
```

默认路径保持旧行为。新 option、config 或 CLI flag 使用 conservative default，并在 help/schema/docs 中同步说明。

## Rollback-Friendly Shape

优先选择易 revert 的形状：

- 先 additive：新增 helper、test、adapter operation branch，再替换旧路径。
- 把 deletion 放在最后一片，并确保替代路径已验证。
- Migration 或 generated fixture change 附带 rollback / regeneration path。
- 一个 commit 对应一个 verified slice，commit message 写清 behavior 和验证。

## Simplicity Check

实现前问：

- 这个 slice 是否需要新 abstraction，还是直接函数/局部 helper 足够？
- 是否已经有三个真实用例支持抽象？
- 是否能用现有 parser、schema、adapter SDK 或 CLI helper 完成？
- 是否在为当前需求交付，而不是为假想扩展铺路？
