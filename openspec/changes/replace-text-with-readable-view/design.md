本 change 的目标是用仓库内版本化 renderer config 驱动的 `readable-view` 替代 document operation 的 `text` 输出模式；本文是未审核临时 design，只存在于 `openspec/changes/replace-text-with-readable-view/`，不改变现有主规范或其它文档。

## Context

当前核心 CLI 和 adapter direct CLI 都有独立 document text 分支。核心 `docnav` 在 `text_result` 中逐 operation 拼接字段，adapter SDK 通过 `DirectTextFormatter` 把展示责任交给格式 adapter；warning、error 和正常结果也有各自文本写入路径。与此同时，`readable-json` 已经提供 documented shape，并被 MCP structuredContent 选为结构化阅读来源。

目标读者包括直接使用 CLI 的人类和 AI、审计 CLI 输出的维护者，以及后续通过 MCP TextContent 阅读结果的客户端。新默认输出必须优先保证字段可见、原始字段位置可追踪和正文可直接阅读，同时不能把它伪装成合法 JSON 或完整机器协议。

## Goals / Non-Goals

**Goals:**

- 让 `readable-view` 成为核心 CLI 和 adapter direct CLI 的 document operation 默认阅读输出。
- 让 readable-view 与 readable-json 从同一个 typed readable result 派生，避免 operation 或 adapter 私有字段拼接。
- 通过仓库内版本化 renderer config 指定 block 字段，保留字段原位置并支持不缩进、不 JSON 转义的 Markdown 正文。
- 支持多个显式 block 字段、嵌套字段和正文包含 marker 字样的情况。
- 在同一交付中删除 document `text` 输出枚举、formatter、模板配置、文档和测试契约。
- 为 legacy `defaults.output: "text"` 提供可执行的配置修复路径。
- 保持 protocol-json、adapter invoke、readable-json schema、ref、pagination、routing 行为和非文档纯文本输出不变。

**Non-Goals:**

- 不把 readable-view 定义为 protocol-json 或 readable-json 的替代机器兼容接口。
- 不追求 Rust renderer 与后续 JavaScript renderer 逐字节一致；跨实现只要求遵守同一语义契约和同一仓库内 renderer config。
- 不根据字符串是否包含换行自动选择 block。
- 不允许用户配置、CLI flag、项目配置、用户配置或 adapter manifest 改变 block 字段集合。
- 不引入 YAML、Hjson、TOML 或通用模板语言。
- 不删除 MCP 协议中的 TextContent 类型；删除的是 Docnav document `text` 输出模式和独立文本渲染语义。
- 不删除 help、version 和其它非文档命令使用的纯文本输出通道。

## Decisions

### 1. Document 输出模式固定为三种

Document operation 的 `--output` 枚举固定为：

```text
readable-view
readable-json
protocol-json
```

省略 `--output` 时使用 `readable-view`。`defaults.output` 只接受以上三个值。`text` 不保留 alias、deprecated value、隐式迁移或 fallback；遇到 CLI `--output text` 或普通 document execution 加载到配置值 `"text"` 时返回输入或配置错误。

该规则只约束 document operation 输出模式。`docnav version`、help 和其它非文档纯文本诊断继续使用独立 `PlainText` 通道，不进入 readable-view/readable-json/protocol-json 枚举。

选择该方案是因为并行保留 document `text` 会继续要求维护 formatter、模板、warning 拼接和测试矩阵，无法达到本 change 的删除目标。

### 2. 使用共享 readable 输出 crate

新增小型共享 Rust crate `docnav-readable`，只拥有阅读输出 DTO 辅助、仓库内 renderer config、readable-view renderer、conformance vectors 和相关错误。它依赖 `serde`/`serde_json`，可以依赖共享 operation/result 类型，但不拥有 adapter routing、文档解析或 protocol envelope。

核心 CLI 和 adapter SDK 都先构造与现有 readable-json schema 一致的 typed readable payload，再序列化为一个完整 JSON value：

```text
operation result / stable error + warnings
  -> typed readable payload
  -> complete readable JSON value
       -> readable-json serializer
       -> readable-view renderer + repo renderer config
```

选择独立 crate 而不是放入 `docnav-protocol`，是为了保持原始协议层和阅读输出层分离；选择共享 crate 而不是在 `docnav` 和 adapter SDK 中复制 renderer，是为了避免格式漂移。

### 3. 仓库内 renderer config 声明 block 字段

每个 readable view kind 对应一个随代码提交的 renderer config 条目。config 是仓库内部 contract，不从 `.docnav` 项目配置、用户配置、环境变量、CLI flag 或 adapter manifest 读取。

示意结构：

```json
{
  "version": 1,
  "views": {
    "read": {
      "blocks": ["/content"]
    },
    "error": {
      "blocks": ["/error", "/guidance"]
    }
  }
}
```

规则：

1. `blocks` 只接受 JSON Pointer，并声明哪些字段外置为 block。
2. 配置中列出的 block pointer 必须对应字符串字段；缺失、非字符串、重复 pointer 或 block identity 冲突必须失败。
3. renderer 不根据换行、长度、content type 或用户配置自动选择 block。
4. readable-view 不承诺 JSON header object key 顺序或多个 block section 的输出顺序；调用方、测试和后续 MCP renderer 必须通过字段名、block pointer 和 byte length 判断语义。

选择仓库内 config 而不是 derive attribute，是为了让 block 字段成为集中可审计材料。选择不规定 key 顺序，是为了避免把阅读输出变成跨语言 canonical serializer，也降低 Rust map 类型、JavaScript object ordering 和未来字段扩展带来的实现负担。

### 4. readable-view 使用版本行、JSON header 和长度定界 block

输出格式版本 1：

```text
@docnav-readable-view/1
{
  "ref": "L5:Guide > Install",
  "content": {
    "$block": "/content",
    "bytes": 42
  },
  "content_type": "text/markdown",
  "cost": "8 lines | 0.4 KB",
  "page": null
}

[block /content bytes=42]
## Install

Run `pnpm install`.
[endblock /content]
```

规则：

1. 第一行固定为 `@docnav-readable-view/1`，明确整个输出不是 JSON。
2. 紧随其后的是一个合法、pretty-printed JSON header。
3. 未标记字段保持与 readable-json 相同的值、数组、对象和业务语义；object key 顺序不属于稳定契约。
4. 被标记字符串在原位置替换为 `{ "$block": "<pointer>", "bytes": <utf8-byte-length> }`。
5. JSON header 后输出 block section；block 起始行为 `[block <pointer> bytes=<n>]`，多个 block 的输出顺序不属于稳定契约。
6. 起始行后的恰好 `<n>` 个 UTF-8 bytes 是字段字符串值。renderer 不增加缩进、不解释 Markdown，也不把正文 marker 字样当作 framing。
7. payload 后若没有换行，renderer 增加一个不属于 payload 的 framing LF，再输出 `[endblock <pointer>]`；若 payload 已以 LF 结束，则直接输出 end marker。
8. block pointer 和 byte length 必须与 header 引用一致。空字符串仍可作为零字节 block 表示。
9. 没有 block 字段时只输出版本行和完整 JSON header，不附加 block section。

长度定界使正文即使包含 `[block ...]`、`[endblock ...]` 或多个相同标题也不会产生结构歧义。readable-view 仍定位为阅读 contract；需要长期机器兼容的调用方继续使用 readable-json 或 protocol-json。

### 5. warning 和 error 先进入 readable payload

CLI ignored argv warning、adapter candidate warning、readable error code/details/guidance 必须先成为完整 readable payload 的字段，再进入 readable-view renderer。renderer 不在 JSON header 或 block section 后额外拼接 warning/error 文本。

这保证 readable-view 不会因为 warning 或错误路径重新出现独立文本模板，也能让审计者从 header 中看到完整结构。

### 6. Renderer 失败必须原子化且不可 fallback

Renderer 必须先在内存中完成 readable-view 渲染，再写 stdout。config 校验、pointer 查找、block 替换或 JSON header 序列化失败时：

1. stdout 不得包含部分 readable-view 输出。
2. stderr 输出稳定诊断，错误 id 使用 `readable_view_render_failed`。
3. 命令返回内部错误 exit code；当前 Rust CLI 和 adapter direct CLI 使用项目现有内部错误退出码。
4. 不得 fallback 到 readable-json、protocol-json、plain text、旧 text formatter 或递归 readable-view 错误渲染。

写 stdout 过程中发生的 I/O 错误继续按项目既有 I/O 错误处理；该类错误不需要伪装成完整 readable-view。

### 7. Legacy text 配置提供专用修复路径

普通 document execution 加载到项目或用户配置中的 `defaults.output: "text"` 时必须失败，并在诊断中包含配置路径、字段路径、legacy value 和修复命令。

配置修复命令必须能在目标 scope 自身包含 legacy text 时运行：

```text
docnav config set defaults.output readable-view
docnav config unset defaults.output
docnav config set --user defaults.output readable-view
docnav config unset --user defaults.output
```

实现上允许为 `config set/unset` 增加 migration-safe raw target config loader。该 loader 只应宽容当前正在修复的目标配置文件中的 legacy output value，不应放宽其它配置字段校验，也不应把 `"text"` 迁移为运行时默认值。

### 8. MCP 归属由 dependent change 承接

当前 change 负责 readable-view 的 Rust 契约、仓库内 renderer config、document CLI 接入和 conformance vectors。`docnav-mcp` 的 JavaScript renderer、TextContent 输出和 bridge wiring 不在当前 change 中实现。

`implement-docnav-mcp-bridge` 必须在开始实现 MCP TextContent 前同步为依赖本 change：

- structuredContent 继续来自 `docnav --output readable-json`。
- TextContent 使用符合本 design 的 readable-view contract。
- bridge 不解析 Markdown，也不自行决定哪些字段成为 block。
- 如果维护 JavaScript renderer，其 conformance 目标是语义一致：格式版本、block pointer、byte length 和 block payload 正确；不要求 header key 顺序、block section 顺序或与 Rust 输出逐字节一致。

这样可以避免当前 change 在 MCP bridge 尚未存在时承担不可执行的 JS 实现任务，同时保证 MCP change 不重新引入旧 text formatter。

### 9. text 删除采用同一变更内原子迁移

实现不经历长期双默认或双 formatter 阶段。可以在开发提交中先引入 renderer，再切换调用方，但 change 验收时必须满足：

- `text` 不在 document operation output enum/help/config 中。
- `OutputMode::Text`、`DirectOutputMode::Text`、`DirectTextFormatter` 和 document text-specific writer/template 均不存在。
- 当前泛用 `CommandOutput::Text` 不再承载 document text output 语义；实现应将非文档纯文本承载重命名为 `PlainText` 或等价明确名称，只用于 help、version 和其它非文档纯文本诊断。
- 代码、主规范、README、测试、skills 和 active change 中不存在把 `text` 当作受支持 document output mode 的当前规则。
- 历史 archive 只作为审计记录保留，不改写。

## Risks / Trade-offs

- [自定义格式增加维护面] -> 保持版本 1 语法最小，只包含版本行、JSON header 和长度定界 block，并用仓库内 renderer config 固定可审计规则。
- [输出顺序被不同语言实现理解不一致] -> readable-view 不把 header key 顺序或多 block 顺序作为稳定契约；跨语言 conformance 验证字段和 block 语义，不验证顺序或逐字节相同。
- [UTF-8 byte length 与字符数混淆] -> 字段名固定为 `bytes`，测试覆盖中文、组合字符、emoji、CRLF 和无尾换行正文。
- [renderer config 与 readable shape 漂移] -> block pointer 缺失或非字符串直接失败；schema/golden/renderer 测试在同一 change 更新，确保外置字段仍能还原到 readable payload 字段。
- [legacy text 配置锁死 config 命令] -> `config set/unset` 使用 migration-safe raw target loader 修复目标配置，同时普通 document execution 严格拒绝 legacy text。
- [删除 text 破坏已有脚本和配置] -> 这是显式 breaking change；help、错误和迁移文档只指向 `readable-view`、`readable-json` 或 `protocol-json`，不提供静默兼容。
- [active change 继续写入旧文本约定] -> 实现前同步 `explore-operation-composition` 和 `implement-docnav-mcp-bridge` 的未完成 artifact 或实现任务；探索 change 不定稿具体文本输出，后续实现 change 必须接入 readable-view。

## Migration Plan

1. 建立 `docnav-readable`、格式版本、仓库内 renderer config 和 conformance vectors。
2. 让核心 CLI 与 adapter SDK 从完整 readable payload 生成 readable-json 和 readable-view。
3. 将 document 默认值、help 和配置枚举切换到 readable-view，并保留非文档 `PlainText` 通道。
4. 实现 legacy `defaults.output: "text"` 的普通执行拒绝和 config set/unset 修复路径。
5. 删除核心和 adapter 的 document text enum、formatter、writer、模板配置和测试。
6. 同步主规范、README、skills、examples/golden、testing matrix 和受影响 active change。
7. 运行局部 Rust/Node 测试、OpenSpec 严格校验和 `pnpm run verify:docnav-workspace`。

回滚只能整体回滚本 change；不支持在已交付状态通过 feature flag 恢复 document `text`。

## Open Questions

无。格式版本、repo-internal renderer config、block pointer、byte framing、默认模式、legacy config 修复路径、MCP 所属 change 和 document text 删除边界均在本 change 中确定。
