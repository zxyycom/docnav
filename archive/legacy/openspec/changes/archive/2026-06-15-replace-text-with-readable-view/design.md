本 change 的目标是用仓库内 renderer config 驱动的 `readable-view` 替代 document operation 的 `text` 输出模式。

## Context

当前核心 CLI 和 adapter direct CLI 都有独立 document text 分支。核心 `docnav` 在 `text_result` 中逐 operation 拼接字段，adapter SDK 通过 `DirectTextFormatter` 把展示责任交给格式 adapter；warning、error 和正常结果也有各自文本写入路径。与此同时，`readable-json` 已经提供 documented shape，并被 MCP structuredContent 选为结构化阅读来源。

目标读者包括直接使用 CLI 的人类和 AI、审计 CLI 输出的维护者，以及后续通过 MCP TextContent 阅读结果的客户端。新默认输出必须优先保证字段可见、原始字段位置可追踪和正文可直接阅读。

## Goals / Non-Goals

**Goals:**

- 让 `readable-view` 成为核心 CLI 和 adapter direct CLI 的 document operation 默认阅读输出。
- 让 readable-view 与 readable-json 从同一个 typed readable result 派生，operation 和 adapter 的展示差异通过 typed readable payload 与 renderer config 表达。
- 通过仓库内 renderer config 指定 block 字段，保留字段原位置并支持不缩进、不 JSON 转义的 Markdown 正文。
- 支持多个显式 block 字段、嵌套字段和正文包含 marker 字样的情况。
- 在同一交付中将 document output surface 收敛到三种当前模式，并更新相关枚举、formatter、模板配置、文档和测试契约。
- 将已有 `defaults.output` 配置 key 的默认值和合法值收敛到 readable-view、readable-json 和 protocol-json。
- 保持 protocol-json、adapter invoke、readable-json schema、ref、pagination、routing 行为和非文档纯文本输出不变。

**Boundaries:**

- protocol-json 和 readable-json 继续承担既有机器可读入口；readable-view 是阅读输出格式。
- Rust renderer 与后续 JavaScript renderer 的 conformance 目标是语义一致：block pointer、byte length 和 block payload 正确。
- block 字段集合只来自仓库内 renderer config，renderer 按声明的 JSON Pointer 选择 block。
- 机器协议格式保持 JSON schema 当前边界；YAML、Hjson、TOML 或通用模板语言属于其它独立设计空间。
- MCP TextContent transport 继续存在；本 change 收敛的是 Docnav document output surface 和独立文本渲染语义。
- help、version 和其它非文档命令继续使用独立纯文本通道。

## Decisions

### 1. Document 输出模式固定为三种

Document operation 的 `--output` 枚举固定为：

```text
readable-view
readable-json
protocol-json
```

省略 `--output` 时使用 `readable-view`。CLI `--output` 和 `defaults.output` 只接受以上三个值；其它 output value 按通用输入错误或配置错误报告。

该规则只约束 document operation 输出模式。`docnav version`、help 和其它非文档纯文本诊断继续使用独立 `PlainText` 通道，不进入 readable-view/readable-json/protocol-json 枚举。

选择该方案是为了让默认阅读输出、warning/error 映射和测试矩阵都走同一 typed readable payload 与 shared renderer path。

### 2. 使用共享 readable 输出 crate

新增小型共享 Rust crate `docnav-readable`，拥有阅读输出 DTO 辅助、仓库内 renderer config、readable-view renderer、conformance vectors 和相关错误。它依赖 `serde`/`serde_json`，可以依赖共享 operation/result 类型；adapter routing、文档解析和 protocol envelope 继续由既有 owner 承担。

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

每个 readable view kind 对应一个随代码提交的 renderer config 条目。config 是仓库内部 contract，由仓库代码拥有；`.docnav` 项目配置、用户配置、环境变量、CLI flag 和 adapter manifest 不承载 block 字段集合。

示意结构：

```json
{
  "views": {
    "read": {
      "blocks": ["/content"]
    },
    "error": {
      "blocks": ["/error"]
    }
  }
}
```

规则：

1. `blocks` 只接受 JSON Pointer，并声明哪些字段外置为 block。
2. 配置中列出的 block pointer 必须对应字符串字段；缺失、非字符串、重复 pointer 或 block identity 冲突必须失败。
3. renderer 只根据 config 中声明的 JSON Pointer 选择 block 字段。
4. readable error 的 `guidance` 等数组字段保持 header JSON 值；只有字符串字段可声明为 block。
5. readable-view 的稳定语义通过字段名、block pointer 和 byte length 判断；conformance 对 JSON header object key 顺序和多个 block section 的输出顺序执行顺序无关断言。

选择仓库内 config 而不是 derive attribute，是为了让 block 字段成为集中可审计材料。header key 和 block section 顺序保持非稳定，避免把阅读输出变成跨语言 canonical serializer，也降低 Rust map 类型、JavaScript object ordering 和未来字段扩展带来的实现负担。

### 4. readable-view 使用 JSON header 和长度定界 block

输出格式：

```text
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

1. 输出从完整、合法、pretty-printed JSON header 开始，header 以 LF byte `0x0A` 结束；readable-view framing 在所有平台使用该字节。
2. 未标记字段保持与 readable-json 相同的值、数组、对象和业务语义；conformance 按字段和值断言 header 语义。
3. 被标记字符串在原位置替换为 `{ "$block": "<pointer>", "bytes": <utf8-byte-length> }`。
4. 存在 block section 时，header 结束 LF 后再写一个空 separator LF，然后输出第一个 block section；没有 block 字段时只输出完整 JSON header。
5. block 起始行为 `[block <pointer> bytes=<n>]`，marker 行以 LF byte `0x0A` 结束；多个 block 的 conformance 通过 pointer 和 byte length 匹配。
6. 起始行后的恰好 `<n>` 个 UTF-8 bytes 是字段字符串值。renderer 按原字段字符串写出 payload；正文中的 LF、CRLF、marker 字样和其它字符都按普通 payload bytes 处理。
7. payload 后若没有 LF 结尾，renderer 增加一个不属于 payload 的 framing LF byte `0x0A`，再输出 `[endblock <pointer>]`；若 payload 已以 LF 结尾，则直接输出 end marker。end marker 行以 LF byte `0x0A` 结束，后续 block 从下一行开始。
8. block pointer 和 byte length 必须与 header 引用一致。空字符串仍可作为零字节 block 表示。

长度定界使正文即使包含 `[block ...]`、`[endblock ...]` 或多个相同标题也不会产生结构歧义。readable-view 仍定位为阅读 contract；需要结构化消费的调用方使用 readable-json，需要 protocol envelope 的调用方使用 protocol-json。

### 5. warning 和 error 先进入 readable payload

CLI ignored argv warning、adapter candidate warning、readable error code/details/guidance 必须先成为完整 readable payload 的字段，再进入 readable-view renderer。warning/error 的阅读语义由 payload、readable-json 和 readable-view header 承载。

这样 warning 和错误路径都通过同一 readable payload/header 表达，审计者可以从 header 中看到完整结构。

### 6. Renderer 失败收束为稳定边界

Renderer 必须先在内存中完成 readable-view 渲染，再写 stdout。config 校验、pointer 查找、block 替换或 JSON header 序列化失败时使用单一路径：

1. stdout 为空。
2. stderr 输出稳定诊断，错误 id 使用 `readable_view_render_failed`。
3. 命令返回内部错误 exit code；当前 Rust CLI 和 adapter direct CLI 使用项目现有内部错误退出码。

写 stdout 过程中发生的 I/O 错误继续按项目既有 I/O 错误处理；该类错误不构造完整 readable-view 错误 payload。

### 7. `defaults.output` 配置校验使用三种当前模式

普通 document execution 加载到项目或用户配置中的 `defaults.output` 时，通过已有配置加载与校验流程确认该值属于三种当前 document output mode。无效值返回配置错误，并在诊断中包含配置路径、字段路径、收到的值和可接受值。

`docnav config set defaults.output <value>` 的合法值与 document output mode 保持一致：

```text
readable-view
readable-json
protocol-json
```

`docnav config set/unset` 使用正常配置加载和 schema 校验流程，并与 document execution 共享三种当前模式校验。

### 8. MCP 归属由 dependent change 承接

当前 change 负责 readable-view 的 Rust 契约、仓库内 renderer config、document CLI 接入和 conformance vectors。`docnav-mcp` 的 JavaScript renderer、TextContent 输出和 bridge wiring 由 `implement-docnav-mcp-bridge` 承接。

`implement-docnav-mcp-bridge` 必须在开始实现 MCP TextContent 前同步为依赖本 change：

- structuredContent 继续来自 `docnav --output readable-json`。
- TextContent 使用符合本 design 的 readable-view contract。
- bridge 消费 core CLI readable output 和仓库 renderer config；Markdown parsing 和 block 字段选择继续由 owning layer 负责。
- 如果维护 JavaScript renderer，其 conformance 目标是语义一致：block pointer、byte length 和 block payload 正确；断言范围聚焦语义字段，而不是 header key 顺序、block section 顺序或与 Rust 输出逐字节一致。

这样让当前 change 只交付可执行的 Rust 契约和 conformance 材料，MCP change 再按同一契约实现 JavaScript renderer 和 TextContent wiring。

#### MCP-consumable contract

以下是 `implement-docnav-mcp-bridge` 必须消费的 contract artifact，由本 change 在 `docnav-readable` crate 中提供：

**A. Renderer config schema**

```json
{
  "views": {
    "<view-kind>": {
      "blocks": ["<json-pointer>", ...]
    }
  }
}
```

- `views` 按 typed readable result kind 索引（`outline`、`read`、`find`、`info`、`error`）。
- 每个 view 的 `blocks` 声明需要外置为 block section 的 JSON Pointer；pointer 必须指向字符串字段。
- 未声明字段保持 header JSON 值（包括多行字符串）。
- renderer config 是仓库内文件，由 Rust 代码加载；JavaScript renderer 消费等价的 config 结构，不要求读取同一文件格式。

**B. Conformance vectors**

跨语言 renderer conformance 必须验证以下语义字段，不要求 header key 顺序、block section 输出顺序或逐字节一致：

| 向量 | 说明 | 断言方式 |
|------|------|----------|
| header 是合法 pretty JSON | JSON.parse 成功，顶层为 object | 解析后按字段名取值 |
| block pointer 存在且正确 | header 中被声明字段的值为 `{"$block": "<pointer>", "bytes": <n>}` | 按字段名取 `$block` 和 `bytes` |
| byte length 精确 | `bytes` 等于该字段字符串的 UTF-8 字节数 | 用 `TextEncoder` 或等价方式计算 |
| block payload 可还原 | `[block <pointer> bytes=<n>]` 起始行后的恰好 `<n>` bytes 等于原字段字符串 | 截取 payload 后与 readable-json 对应字段比对 |
| 多个 block 独立可定位 | 每个 block 的 pointer 和 byte length 与 header 对应 `$block` 引用一致 | 按 pointer 匹配，不依赖输出顺序 |
| 未声明字段保留 | 非 block 字段的值与 readable-json 同名字段一致 | 逐字段比对（除被外置字段外） |
| warning 数组一致 | `warnings` 数组元素和顺序与 readable-json 一致 | 数组深度比对 |
| 空 block 合法 | 零长度字符串输出 `bytes=0` 的 block section | payload 为空，boundary marker 正确 |
| framing LF 一致 | separator 和 marker 行使用 LF byte `0x0A` | 验证 framing bytes，不检查 payload 内部换行 |

**C. 不在本 change 中的内容**

以下保留在 `implement-docnav-mcp-bridge`：
- JavaScript/Node.js renderer 实现。
- MCP TextContent 的 bridge wiring（子进程调用、stdout 读取、TextContent 构造）。
- MCP tool outputSchema 声明。
- MCP error mapping 和 stderr 诊断处理。

### 9. Document output surface 采用同一变更内原子收敛

可以在开发提交中先引入 renderer，再切换调用方；change 验收时满足：

- document operation output enum/help/config 只展示 readable-view、readable-json 和 protocol-json。
- 核心和 SDK document output implementation 走 shared readable payload/renderer path。
- 非文档纯文本承载命名为 `PlainText` 或等价明确名称，只用于 help、version 和其它非文档纯文本诊断。
- 代码、主规范、README、测试、skills 和 active change 与三种当前 document output mode 对齐。
- 归档目录保持审计记录状态。

## Risks / Trade-offs

- [自定义格式增加维护面] -> 保持 readable-view 语法最小，只包含 JSON header、header/block separator 和长度定界 block，并用仓库内 renderer config 固定可审计规则。
- [输出顺序被不同语言实现理解不一致] -> readable-view 的稳定语义使用字段名、block pointer 和 byte length 表达；跨语言 conformance 验证字段和 block 语义。
- [UTF-8 byte length 与字符数混淆] -> 字段名固定为 `bytes`，测试覆盖中文、组合字符、emoji、CRLF、平台无关 framing LF byte 和无尾换行正文。
- [renderer config 与 readable shape 漂移] -> block pointer 缺失或非字符串直接失败；schema/golden/renderer 测试在同一 change 更新，确保外置字段仍能还原到 readable payload 字段。
- [配置枚举与执行路径不一致] -> `defaults.output` 校验返回配置路径、字段路径、收到的值和可接受值；普通 `config set/unset` 写入 readable-view、readable-json 或 protocol-json。
- [脚本、示例或文档未同步到三种模式] -> help、错误诊断、smoke、golden 和最终搜索审计均以 readable-view、readable-json 或 protocol-json 为 document output mode。
- [active change 输出假设分叉] -> 实现前同步 `explore-operation-composition` 和 `implement-docnav-mcp-bridge` 的未完成 artifact 或实现任务；探索 change 保留归属和筛选标准，后续实现 change 按 readable-view contract 定稿。

## Implementation Plan

1. 建立 `docnav-readable`、仓库内 renderer config 和 conformance vectors。
2. 让核心 CLI 与 adapter SDK 从完整 readable payload 生成 readable-json 和 readable-view。
3. 将 document 默认值、help 和配置枚举切换到 readable-view，并保留非文档 `PlainText` 通道。
4. 更新已有 `defaults.output` key 的三种模式校验和配置错误诊断。
5. 将核心和 adapter 的 document output enum、formatter、writer、模板配置和测试收敛到 shared readable payload/renderer path。
6. 同步主规范、README、skills、examples/golden、testing matrix 和受影响 active change。
7. 运行局部 Rust/Node 测试、OpenSpec 严格校验和 `pnpm run verify:docnav-workspace`。

撤销策略按本 change 的原子范围处理；运行时代码只实现 readable-view、readable-json 和 protocol-json 三种当前 document output mode。

## Open Questions

无。repo-internal renderer config、block pointer、byte framing、默认模式、配置校验、MCP 所属 change 和 document output surface 边界均在本 change 中确定。
