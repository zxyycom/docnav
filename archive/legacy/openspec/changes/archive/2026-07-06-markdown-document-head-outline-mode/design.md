本 design 定义 Markdown document head ref 的范围、read/find 行为和验证边界。

## Context

当前 Markdown heading section 从 heading 自身开始，首个有效 heading 前的内容不属于任何 heading ref。`find` 命中这一区域时只能退回 `doc:full`，可以保证 read 包含命中文本，但读取粒度过粗。

Document head 将这段前导内容建模为 Markdown adapter-owned readable region，使 outline 可以提供结构化入口，read 继续承担大段内容和分页输出。

## Goals / Non-Goals

**Goals:**

- 将首个有效 Markdown heading 前的非空内容建模为 document head 可读区域。
- 用一个 `HEAD:leading` entry 暴露 document head，让调用方一次 read 获取 frontmatter 与前导正文原文。
- 让 find 命中 document head 时返回可 read 的区域 ref。
- 保持 heading ref grammar、普通 heading read、`doc:full` fallback、shared ref opacity 和 protocol top-level shape 不变。

**Non-Goals:**

- 不新增 outline 顶层 `frontmatter`、`metadata` 或 `document_head` 字段。
- 不解析 YAML 字段语义，不定义 metadata schema、排序、合并或规范化。
- 不把 document head 当作其它 adapter 必须复用的共享概念。
- 不改变 `outline-unstructured-full-read` 的整篇全文读取策略。

## Decisions

1. **document head 是一个合并可读区域。**

   Document head 定义为当前 Markdown 文档从开头到第一个有效 Markdown heading 起点之前的原文区域。区域读取保留原文，包括 YAML delimiter、空行、注释、序言和其它前导文本，避免 adapter 猜测摘要或 metadata 语义。

   只有 document head 包含非空、非纯空白内容，并且当前 structured outline 至少有一个可见 heading entry 时，outline 才暴露 document head entry。当前 outline 参数下没有可见 heading 时，Markdown outline 继续使用既有 `doc:full` fallback。

2. **满足条件时始终暴露。**

   Document head 是 Markdown structured outline 的默认组成部分。只要满足 entry eligibility，outline 就返回 `HEAD:leading`。

   这个 ref 仍是 adapter-owned opaque string。Core、protocol 和 readable output 只传递或展示 adapter 返回的 ref，不解析 document head 语义。

3. **outline entry 只作为导航入口，内容通过 read 分页。**

   Outline 中的 `HEAD:leading` entry 只返回既有 entry facts，例如 `ref`、非空 `label`、`kind`、`location.line_start` 和必要 metadata。大段内容仍由 read 返回，并沿用 read 的 Unicode 字符预算、page 和 content block 输出规则。Raw protocol 不返回 `display`；readable output 从 entry facts 派生 display。

4. **find 必须返回可继续 read 的区域 ref。**

   当 find 命中 document head 且当前 structured outline 至少有一个可见 heading entry 时，match ref 使用 `HEAD:leading`。当当前 outline 走 `doc:full` fallback 时，find 保持可 read 的既有 fallback 行为，必要时使用 `doc:full`。

## Risks / Trade-offs

- [Risk] structured outline 默认多一个非 heading entry。Mitigation: entry 使用非 heading `kind`、清晰 `label` 和不同 ref prefix，调用方仍可按 `kind` 或 ref prefix 区分 heading 与 document head。

## Implementation Order

1. 同步 `docs/adapters/markdown.md`、testing case ledger 和测试计划。
2. 实现 parser 区域识别、outline/find/read ref 和 smoke/unit tests。
3. 运行 adapter tests、smoke tests、OpenSpec strict validation 和 workspace verification。
