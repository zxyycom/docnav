# MarkdownNavigator 参考基线

MarkdownNavigator 只用于记录外部来源和迁移决策，不是现行 Markdown adapter 行为规范。现行 Markdown 导航行为以 [Markdown Adapter](../adapters/markdown.md) 为准。

本节中出现的旧 heading ref 格式（`L{line}:{path}`、`L{line}#{ordinal}:{path}`）仅为历史迁移记录。当前 canonical heading ref 格式为 `H:L{line}:H{level}:I{index}`，旧格式不再可用。

## 来源与复验

| 项目 | 值 |
| --- | --- |
| 仓库路径 | `D:\project\skills\MarkdownNavigator` |
| 参考提交 | `714605078b0158975dbd16b8c739ad3b41098978` |
| 参考版本 | `markdown-navigator 0.1.0` |
| parser | `pulldown-cmark 0.13.4` |
| 复验命令 | `uv run python tests/code/main.py --profile debug` |

复验应在参考仓库内针对上述提交和命令入口执行，用于确认本文件记录的 Markdown 行为边界。长期参考文档只记录可复用来源、命令入口和迁移决策，不记录单次运行日期或一次性结果。

## 迁移决策

| 行为 | 参考项目现状 | 决策 | Docnav v0 |
| --- | --- | --- | --- |
| heading 识别 | 使用成熟 parser | 保留 | 使用成熟 parser |
| 章节范围 | heading 到下一个同级或更高级 heading 前 | 保留 | read 保持相同范围 |
| frontmatter | 不生成 heading | 保留 | 不进入 outline |
| 代码围栏伪 heading | 不生成 heading | 保留 | 不进入 outline |
| 深层 heading | 可解析 H1-H6 | 保留 | parser 保留 H1-H6 |
| 默认展示级别 | 默认 H1-H3 | 调整 | `docnav-markdown` 内置 `max_heading_level: 3` |
| 无可见 outline | 可返回空 heading 集合 | 调整 | 返回一个全文 ref entry |
| headings 输出 | columns 与位置数组 | 移除 | 扁平 `ref + display` entries |
| 自由文本 path/heading | 可选择章节 | 移除 | 使用 adapter 生成的结构 ref（`H:L{line}:H{level}:I{index}`） |
| 重复标题和路径 | line hint 可最近匹配 | 调整 | 每项按 line/level/index 生成不同 ref；read 禁止最近位置消歧；旧 breadcrumb ref 不可用 |
| 编码 | 可显式使用非 UTF-8 | 推迟 | v0 只支持 UTF-8 |
| 默认分页 | 旧项目使用预览限制和 stderr 提示 | 调整 | `limit_chars` 字符预算和 page |
| section 裸文本 | stdout 只输出内容 | 调整 | 默认 CLI 输出可读内容；protocol-json 使用 envelope |
| 非结构化错误 | stderr 文案 | 移除 | 原始协议使用稳定结构化错误 |
| 未知参数 | 警告后继续 | 调整 | Docnav 直接 CLI warning 后继续；`invoke` stdin JSON 仍严格失败 |

## Markdown v0 默认值

- outline 每页最多返回 6000 字符。
- outline 默认只展示 H1-H3。
- read 每页最多返回 6000 字符。
- find 每页最多返回 6000 字符。
- find 默认使用 H1-H3 outline entries 选择 match ref 归属。

默认值属于 `docnav-markdown` 配置域，可被该 CLI 的项目级或用户级配置覆盖。invoke 请求必须显式携带最终值。

## 扁平 Outline

Markdown outline 按文档顺序返回扁平条目。层级关系通过 display 中的 breadcrumb 表达，不生成通用树字段。当前 ref 使用 `H:L{line}:H{level}:I{index}` 格式（详见 [Markdown Adapter](../adapters/markdown.md)）。

```text
# 现行格式示例（H:L{line}:H{level}:I{index}）
H:L1:H1:I1   Guide                     | 9 lines | 0.1 KB
H:L4:H2:I2   Guide > Install           | 6 lines | 0.1 KB
H:L7:H3:I3   Guide > Install > Windows | 3 lines | 34 B
```

历史格式（`L1:Guide`、`L4:Guide > Install` 等）已不可用，仅作为非法 grammar 测试输入保留。

若当前 outline 参数过滤后没有任何 entry，outline 返回一个全文 ref entry。该 ref 读取整篇 Markdown 文档；它覆盖无 heading 文档，也覆盖默认 H1-H3 下只有 H4-H6 的文档。

## Find Ref 归属

Markdown find 搜索全文，但 match ref 指向当前 outline 参数下离命中位置最近的 outline entry，而不是默认归到全文 ref。若当前 outline 参数没有任何 entry，find 使用全文 ref。

最近 outline 按源码位置判断；命中位于两个 outline entry 之间时选择距离更近的一项，距离相同则选择前一项，保证结果确定。

## 章节范围基线

1. 目标章节从目标 heading 开始。
2. 更深层 heading 属于目标章节。
3. 章节在下一个同级或更高级 heading 前结束。
4. 代码围栏伪 heading 不影响范围。
5. frontmatter 不产生 heading。

## Fixtures 与黑盒场景

Markdown 适配器测试至少覆盖：

- `basic.md`：嵌套 heading、代码围栏、扁平 display。
- `deep_sections.md`：深层章节范围。
- `duplicate_headings.md`：重复标题唯一 ref。
- `duplicate_paths.md`：重复完整路径唯一 ref，禁止最近位置消歧。
- `frontmatter.md`：frontmatter 不进入 outline。
- `invalid_headings.md`：无效 heading 被忽略。
- `no_headings.md`：返回全文 ref entry，并可 read 整篇文档。
- `only_deep.md`：默认 H1-H3 返回全文 ref entry，显式参数覆盖后返回深层 heading entries。
- `find_nearest_outline.md`：match ref 指向最近 outline，outline 为空时使用全文 ref。
- `gb18030.md`：v0 返回编码不支持。

必须额外验证 ref 在当前文档中唯一、全文 ref fallback、find 最近 outline 归属、字符预算默认值和 page 分页。
