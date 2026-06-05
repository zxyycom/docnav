# MarkdownNavigator 参考基线

MarkdownNavigator 只用于验证 Markdown 产品行为，不是 Docnav 协议、ref、CLI 或输出兼容目标。

## 来源与验证

| 项目 | 值 |
| --- | --- |
| 仓库路径 | `D:\project\skills\MarkdownNavigator` |
| 参考提交 | `714605078b0158975dbd16b8c739ad3b41098978` |
| 参考版本 | `markdown-navigator 0.1.0` |
| parser | `pulldown-cmark 0.13.4` |
| 验证时间 | `2026-06-04`（Asia/Shanghai） |
| 验证命令 | `uv run python tests/code/main.py --profile debug` |
| 验证结果 | 41 个黑盒场景全部通过 |

## 迁移决策

| 行为 | 参考项目现状 | 决策 | Docnav v0 |
| --- | --- | --- | --- |
| heading 识别 | 使用成熟 parser | 保留 | 使用成熟 parser |
| 章节范围 | heading 到下一个同级或更高级 heading 前 | 保留 | read 保持相同范围 |
| frontmatter | 不生成 heading | 保留 | 不进入 outline |
| 代码围栏伪 heading | 不生成 heading | 保留 | 不进入 outline |
| 深层 heading | 可解析 H1-H6 | 保留 | parser 保留 H1-H6 |
| 默认展示级别 | 默认 H1-H3 | 调整 | `docnav-markdown` 内置 `max_heading_level: 3` |
| headings 输出 | columns 与位置数组 | 移除 | 扁平 `ref + display` entries |
| 自由文本 path/heading | 可选择章节 | 移除 | 使用适配器生成的可读 ref |
| 重复标题和路径 | line hint 可最近匹配 | 调整 | 每项生成唯一 ref；read 禁止最近位置消歧 |
| 编码 | 可显式使用非 UTF-8 | 推迟 | v0 只支持 UTF-8 |
| 默认分页 | 旧项目使用预览限制和 stderr 提示 | 调整 | `limit_chars` 字符预算和 page |
| section 裸文本 | stdout 只输出内容 | 调整 | 默认 CLI 输出可读内容；protocol-json 使用 envelope |
| 非结构化错误 | stderr 文案 | 移除 | 原始协议使用稳定结构化错误 |
| 未知参数 | 警告后继续 | 移除 | Docnav CLI 必须失败 |

## Markdown v0 默认值

- outline 每页最多返回 6000 字符。
- outline 默认只展示 H1-H3。
- read 每页最多返回 6000 字符。
- find 每页最多返回 6000 字符。

默认值属于 `docnav-markdown` 配置域，可被该 CLI 的项目级或用户级配置覆盖。invoke 请求必须显式携带最终值。

## 扁平 Outline

Markdown outline 按文档顺序返回扁平条目。层级关系只通过 Markdown 有意义的 heading path 和 ref locator 表达，不生成通用树字段。

```text
L1:Guide                     | 9 lines | 0.1 KB
L4:Guide > Install           | 6 lines | 0.1 KB
L7:Guide > Install > Windows | 3 lines | 34 B
```

## 章节范围基线

1. 目标章节从目标 heading 开始。
2. 更深层 heading 属于目标章节。
3. 章节在下一个同级或更高级 heading 前结束。
4. 代码围栏伪 heading 不影响范围。
5. frontmatter 不产生 heading。

## Fixtures 与黑盒场景

后续 Markdown 适配器至少覆盖：

- `basic.md`：嵌套 heading、代码围栏、扁平 display。
- `deep_sections.md`：深层章节范围。
- `duplicate_headings.md`：重复标题唯一 ref。
- `duplicate_paths.md`：重复完整路径唯一 ref，禁止最近位置消歧。
- `frontmatter.md`：frontmatter 不进入 outline。
- `invalid_headings.md`：无效 heading 被忽略。
- `no_headings.md`：空 outline 或明确结果。
- `only_deep.md`：默认 H1-H3 过滤与显式参数覆盖。
- `gb18030.md`：v0 返回编码不支持。

必须额外验证 ref 在当前文档中唯一、字符预算默认值和 page 分页。
