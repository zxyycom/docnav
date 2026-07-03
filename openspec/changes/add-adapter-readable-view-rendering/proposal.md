## Why

本 change 目标是在 `readable-view` 中允许 adapter 可选注入自定义文本渲染，并首先让 Markdown adapter 选择 md-like 的 outline/read/find/info 输出；当前文档只在 `openspec/changes/add-adapter-readable-view-rendering/` 下形成未审核临时文档，不影响现有其它文档或主规范。

当前 `readable-view` 使用 pretty JSON header 加 block framing，适合稳定校验，但默认阅读体验无法覆盖所有格式和 adapter 的展示偏好。通用 hook 的目的不是规定“原格式 like”或“省略语义”，而是给 adapter 一个可选 presentation hook，让 adapter 在不影响机器输出的前提下自行整理 `readable-view` 文本；Markdown 的 md-like 省略输出只是首个使用该 hook 的 adapter 选择。

## What Changes

- 为 document operation 的 `readable-view` 增加 adapter 可选 renderer hook：adapter 可接收已生成的 operation 成功结果和渲染上下文，自行返回一段自定义纯文本。
- 保留 core generic `readable-view` 作为默认 fallback；未实现 hook 的 adapter 继续使用现有或后续统一的基础 readable-view。
- 规定 adapter-rendered `readable-view` 只影响 `readable-view`，不得改变 `protocol-json`、`readable-json`、operation success payload、error envelope、ref、pagination 或 adapter routing。
- 为 Markdown adapter 增加 md-like readable-view：outline 输出 Markdown-like 结构骨架，read 输出 Markdown content/page 投影，find 输出命中上下文的 Markdown-like 片段，info 输出 Markdown-readable 摘要。
- Markdown md-like readable-view 必须用显式省略标记表达被省略的 sibling、children、content、matches 或 page 内容，并保持 ref/continuation guidance 可发现。
- 非目标：不要求通用 adapter renderer hook 实现原格式 like 输出、省略标记、ref guidance 或 continuation guidance；这些语义只在具体 adapter spec 明确要求时成立。
- 非目标：不要求其它格式 adapter 同步实现 md-like 或 native-like readable-view，不把 adapter renderer hook 暴露为用户配置、项目配置、环境变量或 CLI flag。
- 非目标：不要求 adapter-rendered `readable-view` 可被原格式 parser 解析，也不要求它符合原文档业务 schema，除非具体 adapter 自己声明此类约束。

## Capabilities

### New Capabilities
- 无。

### Modified Capabilities
- `readable-view-output`: 修改 `readable-view` 的 ownership 与渲染路径，允许 adapter 可选返回自定义纯文本 readable-view，并规定 generic fallback、失败处理和与 machine outputs 的边界。
- `markdown-navigation`: 增加 Markdown adapter 自己选择的 md-like `readable-view` 行为，覆盖 outline、read、find 和 info 的格式感知省略输出。

## Impact

- 影响 `docnav-output` / `docnav-readable` 的 `readable-view` 编排与 fallback 选择。
- 影响 linked adapter contract 或 adapter operation result 到 readable-view renderer 的内部接口；该 hook 是 adapter capability，不是用户可配置策略。
- 影响 `docnav-markdown` 的 readable-view presentation code 和 Markdown CLI smoke/golden expectations。
- 影响 `docs/output.md`、`docs/adapter-contract.md`、`docs/adapters/markdown.md` 及对应 schema/example/fixture/test 材料，前提是实现阶段确认这些材料当前拥有该 surface。
- 不影响 `protocol-json` 和 `readable-json` 的 JSON schema、raw protocol envelope、ref ownership、adapter selection、navigation input resolution 或 error code 映射。
