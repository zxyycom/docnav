# 来源规则 (Source Rules)

## Source Hierarchy

按以下顺序选择 primary source：

1. Official documentation：API reference、guide、manual、spec section。
2. Official blog、changelog、release notes、migration guide。
3. Web standards references：MDN、WHATWG、W3C、TC39、web.dev 等标准或平台材料。
4. Browser/runtime compatibility：caniuse、Node.js docs、runtime release docs、compat tables。

非 primary source：

1. Stack Overflow、GitHub issues、Reddit、Q&A。
2. 个人 blog、tutorial、course notes。
3. AI-generated docs、search summaries、copied snippets。
4. 训练记忆或“我记得”。

这些材料可以作为线索，但不能作为关键实现决策的最终依据。

## Citation Format

合格 citation：

1. 使用完整 URL。
2. 优先 deep link 或 anchor，指向具体 API、section、migration note 或 compatibility table。
3. 在最终回复中说明该 source 支撑了哪个决策。
4. 对非显而易见、容易过期或安全相关的选择，在代码注释中保留短 source URL。

避免只引用 homepage、docs 根目录或搜索结果页。

## Conflict Handling

当来源冲突时：

1. 先确认项目使用版本。
2. 优先适用于当前版本的 API reference 或 migration guide。
3. 如果 official docs 与 existing code style 冲突，说明两者差异和影响。
4. 如果无法判断，停下来问用户，不把猜测写成事实。

示例输出结构：

```text
Source conflict:
- Current project: React 19.x
- API reference recommends pattern A
- Existing code uses pattern B
- Impact: A matches current docs; B preserves local consistency
Decision needed: choose A for modernization or B for consistency.
```

## Unverified Claims

当找不到 official source 时，用明确标签：

```text
UNVERIFIED: I could not find official documentation for this pattern.
Use only after manual validation.
```

不要用 “probably”、“I think” 或 “should work” 掩盖缺口。无法验证的信息要么移出实现，要么成为显式风险。
